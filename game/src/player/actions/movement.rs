use thecurse_core::creatures::player::{
    AERIAL_MOVEMENT_FACTOR, AerialState, AttackState, MOVEMENT_SPEED, MovementState,
};

use crate::{player::PlayerCharacterHandle, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (movement_input, movement_changes.after(movement_input)).run_if(in_state(AppState::Game)),
    );
}

const MOVEMENT_ANIMATION_SPEED: f32 = 1. + MOVEMENT_SPEED * 0.1;

fn movement_input(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut udp: ResMut<Udp>,
    mut commands: Commands,
    query: Query<
        (
            &mut MovementState,
            &AerialState,
            &AttackState,
            &GltfAnimationTarget,
            &mut Transform,
            &mut LinearVelocity,
        ),
        (With<MainCharacter>, Without<CameraController>),
    >,
    mut armatures: Query<
        &mut Transform,
        (
            With<AnimationPlayer>,
            Without<MainCharacter>,
            Without<CameraController>,
        ),
    >,
    camera: Single<&Transform, With<CameraController>>,
) {
    for (mut moving, aerial, attack, target, mut transform, mut velocity) in query {
        let mut direction = Vec3::ZERO;
        if input.pressed(KeyCode::KeyW) {
            direction.z -= 1.;
        }
        if input.pressed(KeyCode::KeyS) {
            direction.z += 1.;
        }
        if input.pressed(KeyCode::KeyA) {
            direction.x -= 1.;
        }
        if input.pressed(KeyCode::KeyD) {
            direction.x += 1.;
        }

        if direction == Vec3::ZERO || *attack != AttackState::None {
            if moving.base_direction != Vec3::ZERO {
                moving.base_direction = Vec3::ZERO;
                velocity.x = 0.;
                velocity.z = 0.;
                moving.propagation_timer.tick(time.delta());
                udp.write_action(PlayerAction::Movement {
                    origin: moving.last_pos.xz().to_array(),
                    direction: moving.last_propagated_dir.unwrap().to_array(),
                    destination: transform.translation.xz().to_array(),
                    duration_secs: moving.propagation_timer.elapsed_secs(),
                });
                moving.last_propagated_dir = None;
                moving.last_pos = transform.translation;
                moving.propagation_timer.reset();
            }
            continue;
        }

        let past = transform.rotation;
        let forward =
            Quat::from_rotation_arc(Vec3::NEG_Z, camera.translation.with_y(0.).normalize());
        transform.rotation = forward;
        transform.rotate_y((-direction.x).atan2(-direction.z));

        if direction != moving.base_direction {
            // Rotate the inner armature entity that is the root of the character mesh back to the
            // rotation the player had previously, then catch up smoothly.
            // Directly animating the character rotation would logically make more sense, but I
            // dislike the input delay it brings (player presses left, but the character walks a
            // curve while smoothly rotating to the left).
            if let Ok(mut armature_transform) = armatures.get_mut(**target) {
                armature_transform.rotation = Quat::from_rotation_arc(
                    transform.rotation * Vec3::NEG_Z,
                    past * armature_transform.rotation * Vec3::NEG_Z,
                );

                commands.entity(**target).transition(Quat::IDENTITY, 100);
            }
            moving.base_direction = direction;
        }

        direction = (camera.rotation * direction).with_y(0.).normalize() * MOVEMENT_SPEED;

        if moving.last_propagated_dir.is_none() {
            moving.last_propagated_dir = Some(direction.xz());
        }

        if *aerial != AerialState::Grounded {
            direction *= AERIAL_MOVEMENT_FACTOR;
        }

        moving.propagation_timer.tick(time.delta());
        if moving.propagation_timer.just_finished()
            || moving
                .last_propagated_dir
                // Send update direction changed by at least 7.5°
                .is_some_and(|last| last.angle_to(direction.xz()).abs() > PI / 24.)
        {
            udp.write_action(PlayerAction::Movement {
                origin: moving.last_pos.xz().to_array(),
                direction: direction.xz().to_array(),
                destination: transform.translation.xz().to_array(),
                duration_secs: moving.propagation_timer.elapsed_secs(),
            });
            moving.last_propagated_dir = Some(direction.xz());
            moving.last_pos = transform.translation;
        }

        velocity.x = direction.x;
        velocity.z = direction.z;
    }
}

fn movement_changes(
    changed: Query<
        (
            &MovementState,
            &AerialState,
            &AttackState,
            &GltfAnimationTarget,
        ),
        Changed<MovementState>,
    >,
    mut players: Query<(&mut AnimationTransitions, &mut AnimationPlayer)>,
    character: Res<PlayerCharacterHandle>,
) {
    for (movement, aerial, attack, target) in changed {
        if *aerial == AerialState::Grounded
            && *attack == AttackState::None
            && let Ok((mut transitions, mut player)) = players.get_mut(**target)
        {
            if movement.base_direction == Vec3::ZERO {
                transitions.play(&mut player, character.idle, Duration::from_millis(100));
            } else {
                transitions
                    .play(&mut player, character.running, Duration::from_millis(100))
                    .set_speed(MOVEMENT_ANIMATION_SPEED)
                    .repeat();
            }
        }
    }
}

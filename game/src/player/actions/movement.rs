use thecurse_core::creatures::player::{
    AERIAL_MOVEMENT_FACTOR, AerialState, AttackState, MOVEMENT_SPEED, MovementState,
};

use crate::{player::PlayerCharacterHandle, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedLast,
        (
            movement_input,
            movement_correction,
            movement_changes.after(movement_input),
        )
            .run_if(in_state(AppState::Game)),
    );
}

const MOVEMENT_ANIMATION_SPEED: f32 = 1. + MOVEMENT_SPEED * 0.1;

fn movement_input(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut commands: Commands,
    query: Query<
        (
            &mut MainCharacter,
            &mut MovementState,
            &AerialState,
            &AttackState,
            &GltfAnimationTarget,
            &mut Transform,
            &mut LinearVelocity,
        ),
        Without<CameraController>,
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
    for (mut character, mut movement, aerial, attack, target, mut transform, mut velocity) in query
    {
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
            if movement.base_direction != Vec3::ZERO {
                movement.base_direction = Vec3::ZERO;
                velocity.x = 0.;
                velocity.z = 0.;
                movement.propagation_timer.tick(time.delta());

                let duration_millis = movement.propagation_timer.elapsed().as_millis() as u8;
                if duration_millis == 0 {
                    movement.last_propagated_dir = None;
                    transform.translation = movement.last_pos;
                    continue;
                }
                let action = PlayerAction::Movement {
                    direction: (transform.translation - movement.last_pos).xz().to_array(),
                    duration_millis,
                };
                character.add_action(super::CachedPlayerAction::Movement {
                    action,
                    motion: transform.translation - movement.last_pos,
                });

                movement.last_propagated_dir = None;
                movement.last_pos = transform.translation;
                movement.propagation_timer.reset();
            }
            continue;
        }

        let past = transform.rotation;
        let forward =
            Quat::from_rotation_arc(Vec3::NEG_Z, camera.translation.with_y(0.).normalize());
        transform.rotation = forward;
        transform.rotate_y((-direction.x).atan2(-direction.z));

        if direction != movement.base_direction {
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
            movement.base_direction = direction;
        }

        direction = (camera.rotation * direction).with_y(0.).normalize() * MOVEMENT_SPEED;

        if movement.last_propagated_dir.is_none() {
            movement.last_propagated_dir = Some(direction.xz());
        }

        if *aerial != AerialState::Grounded {
            direction *= AERIAL_MOVEMENT_FACTOR;
        }

        movement.propagation_timer.tick(time.delta());
        if movement.propagation_timer.just_finished()
            || movement
                .last_propagated_dir
                // Send update direction changed by at least 7.5°
                .is_some_and(|last| last.angle_to(direction.xz()).abs() > PI / 24.)
        {
            let duration_millis = movement.propagation_timer.elapsed().as_millis() as u8
                + movement.propagation_timer.times_finished_this_tick() as u8
                    * movement.propagation_timer.duration().as_millis() as u8;
            if duration_millis == 0 {
                debug_assert!(false);
            }
            let action = PlayerAction::Movement {
                direction: (transform.translation - movement.last_pos).xz().to_array(),
                duration_millis,
            };
            character.add_action(super::CachedPlayerAction::Movement {
                action,
                motion: transform.translation - movement.last_pos,
            });

            movement.last_propagated_dir = Some(direction.xz());
            movement.last_pos = transform.translation;
        }

        velocity.x = direction.x;
        velocity.z = direction.z;
    }
}

fn movement_correction(
    time: Res<Time>,
    query: Query<(&mut MainCharacter, &MovementState, &mut Transform)>,
) {
    for (mut character, movement, mut pos) in query {
        if character.correction_progress < 1. {
            let tick = time.delta_secs() * 5.;
            if character.correction_progress == 0. {
                character.correction = character.authoritative_translation
                    - (movement.last_pos - character.predicted_movement);
            }
            character.correction_progress += tick;
            if character.correction_progress >= 1. {
                pos.translation +=
                    character.correction * (tick - (character.correction_progress - 1.));
                character.correction = Vec3::ZERO;
            } else {
                pos.translation += character.correction * tick;
            }
        }
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

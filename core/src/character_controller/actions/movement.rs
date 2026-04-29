use crate::{
    actions::attack::AttackState, character_controller::actions::aerial::AerialState, prelude::*,
};

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(
            Update,
            (movement_input, movement_changes.after(movement_input)).run_if(in_state(game_state)),
        );
    }
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct MovementState {
    pub direction: Vec3,
}

const MOVEMENT_SPEED: f32 = 10.;
const MOVEMENT_ANIMATION_SPEED: f32 = 1. + MOVEMENT_SPEED * 0.1;

fn movement_input(
    input: Res<ButtonInput<KeyCode>>,
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
        let any_input =
            input.any_pressed([KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD]);

        if !any_input {
            if moving.direction != Vec3::ZERO {
                moving.direction = Vec3::ZERO;
                velocity.x = 0.;
                velocity.z = 0.;
            }
            return;
        }

        let mut dir = Vec3::ZERO;
        if input.pressed(KeyCode::KeyW) {
            dir.z -= 1.;
        }
        if input.pressed(KeyCode::KeyA) {
            dir.x -= 1.;
        }
        if input.pressed(KeyCode::KeyS) {
            dir.z += 1.;
        }
        if input.pressed(KeyCode::KeyD) {
            dir.x += 1.;
        }

        if dir == Vec3::ZERO || *attack != AttackState::None {
            velocity.x = 0.;
            velocity.z = 0.;
            if moving.direction != Vec3::ZERO {
                moving.direction = Vec3::ZERO;
            }
            return;
        }

        if *aerial != AerialState::Grounded {
            dir *= 0.5;
        }

        let past = transform.rotation;
        let forward =
            Quat::from_rotation_arc(Vec3::NEG_Z, camera.translation.with_y(0.).normalize());
        transform.rotation = forward;
        transform.rotate_y((-dir.x).atan2(-dir.z));

        if dir != moving.direction {
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
            moving.direction = dir;
        }

        dir = camera.rotation * dir.normalize() * MOVEMENT_SPEED;

        if *aerial != AerialState::Grounded {
            dir *= 0.5;
        }

        velocity.x = dir.x;
        velocity.z = dir.z;
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
            if movement.direction == Vec3::ZERO {
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

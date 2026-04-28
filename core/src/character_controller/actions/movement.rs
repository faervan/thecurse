use crate::{character_controller::actions::aerial::AerialState, prelude::*};

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

fn movement_input(
    input: Res<ButtonInput<KeyCode>>,
    query: Query<
        (
            &mut MovementState,
            &AerialState,
            &mut Transform,
            &mut LinearVelocity,
        ),
        Without<CameraController>,
    >,
    camera: Single<&Transform, With<CameraController>>,
) {
    for (mut moving, aerial, mut transform, mut velocity) in query {
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

        if *aerial != AerialState::Grounded {
            dir *= 0.5;
        }

        if dir != moving.direction {
            moving.direction = dir;
        }

        if dir == Vec3::ZERO {
            return;
        }

        transform.rotation =
            Quat::from_rotation_arc(Vec3::NEG_Z, camera.translation.with_y(0.).normalize());
        transform.rotate_y((-dir.x).atan2(-dir.z));

        dir = camera.rotation * dir.normalize() * 20.;

        if *aerial != AerialState::Grounded {
            dir *= 0.5;
        }

        velocity.x = dir.x;
        velocity.z = dir.z;
    }
}

fn movement_changes(
    changed: Query<(&MovementState, &AerialState, &GltfAnimationTarget), Changed<MovementState>>,
    mut players: Query<(&mut AnimationTransitions, &mut AnimationPlayer)>,
    character: Res<PlayerCharacterHandle>,
) {
    for (movement, aerial, target) in changed {
        if let Ok((mut transitions, mut player)) = players.get_mut(**target)
            && *aerial == AerialState::Grounded
        {
            if movement.direction == Vec3::ZERO {
                transitions.play(&mut player, character.idle, Duration::from_millis(100));
            } else {
                transitions
                    .play(&mut player, character.running, Duration::from_millis(100))
                    .repeat();
            }
        }
    }
}

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, movement_input);
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct Moving {
    pub direction: Vec3,
}

fn movement_input(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    query: Query<(Entity, Option<&mut Moving>, &mut LinearVelocity)>,
    camera: Single<&Transform, With<Camera3d>>,
) {
    for (entity, mut moving, mut velocity) in query {
        let any_input =
            input.any_pressed([KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD]);

        if !any_input {
            if moving.is_some() {
                moving.take();
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

        if let Some(mut moving) = moving {
            if dir != moving.direction {
                moving.direction = dir;
            }
        } else {
            commands.entity(entity).insert(Moving { direction: dir });
        }

        dir = camera.rotation * dir.normalize_or_zero() * 20.;
        velocity.x = dir.x;
        velocity.z = dir.z;
    }
}

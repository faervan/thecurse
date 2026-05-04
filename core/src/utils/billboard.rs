use crate::prelude::*;

pub(crate) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(
            Update,
            (billboard, billboard_children).run_if(in_state(game_state)),
        );
    }
}

#[derive(Component)]
pub struct Billboarded;

fn billboard(
    query: Query<&mut Transform, (With<Billboarded>, Without<ChildOf>)>,
    camera: Single<&Transform, (With<CameraController>, Without<Billboarded>)>,
) {
    for mut transform in query {
        transform.rotation = -camera.rotation;
    }
}

fn billboard_children(
    query: Query<(&mut Transform, &GlobalTransform, &ChildOf), With<Billboarded>>,
    parents: Query<&GlobalTransform>,
    camera: Single<&GlobalTransform, (With<CameraController>, Without<Billboarded>)>,
) {
    let camera_pos = camera.translation();

    for (mut transform, global_transform, parent) in query {
        let global_pos = global_transform.translation();
        let direction = (camera_pos - global_pos)
            .with_y(0.)
            .normalize_or(Vec3::NEG_Z);

        let world_rot = Quat::from_rotation_arc(Vec3::NEG_Z, direction);

        let parent_global = parents.get(parent.0).unwrap();
        transform.rotation = parent_global.rotation().inverse() * world_rot;
    }
}

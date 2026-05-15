use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, billboard.run_if(in_state(AppState::Game)));
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

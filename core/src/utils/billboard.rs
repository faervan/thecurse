use crate::prelude::*;

pub(crate) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(Update, billboard.run_if(in_state(game_state)));
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

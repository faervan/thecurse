use crate::prelude::*;

pub mod health;

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_plugins(health::plugin(game_state));

        app.add_systems(OnEnter(game_state), test);
    }
}

fn test(mut commands: Commands) {
    commands.spawn((
        Name::new("Test dummy"),
        Health(20.),
        Transform::from_xyz(3., 1., 1.),
        Collider::cuboid(0.8, 2., 0.5),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
    ));
}

use crate::prelude::*;

pub fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(OnEnter(game_state), setup);
    }
}

#[derive(Component)]
pub struct Obstacle;

fn setup(mut commands: Commands) {
    let half_width = 50.;
    let half_height = 50.;
    commands.spawn((
        ManagedNavMesh::from_id(0),
        NavMeshSettings {
            // Define the outer borders of the navmesh.
            fixed: Triangulation::from_outer_edges(&[
                vec2(-half_width, -half_height),
                vec2(half_width, -half_height),
                vec2(half_width, half_height),
                vec2(-half_width, half_height),
            ]),
            build_timeout: Some(1.0),
            simplify: 0.005,
            merge_steps: 0,
            agent_radius: 0.7,
            default_search_delta: 0.1,
            default_search_steps: 8,
            ..default()
        },
        NavMeshUpdateMode::Direct,
        Transform::from_xyz(0.0, 0.1, 0.0).with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
        GameEntity,
    ));
}

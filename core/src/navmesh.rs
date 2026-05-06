use crate::prelude::*;

pub fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(OnEnter(game_state), setup);

        app.add_systems(Update, test.run_if(in_state(game_state)));
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
            ..default()
        },
        NavMeshUpdateMode::Direct,
        Transform::from_xyz(0.0, 0.1, 0.0).with_rotation(Quat::from_rotation_x(FRAC_PI_2)),
    ));
}

fn test(
    mut gizmos: Gizmos,
    navmeshes: Res<Assets<NavMesh>>,
    player: Query<&Transform, With<MainCharacter>>,
) {
    let Some(navmesh) = navmeshes.get(ManagedNavMesh::from_id(0)) else {
        return;
    };
    let Ok(player_transform) = player.single() else {
        return;
    };

    let start = vec2(
        player_transform.translation.x,
        player_transform.translation.z,
    );
    let end = vec2(0., 0.);

    if let Some(mut path) = navmesh.path(start, end) {
        path.path.insert(0, start);
        gizmos.linestrip(
            path.path.into_iter().map(|v| vec3(v.x, 0.1, v.y)),
            bevy::color::palettes::css::BLUE,
        );
    }
}

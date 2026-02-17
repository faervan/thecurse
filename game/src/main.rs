use thecurse_core::{CameraControllerPlugin, CharacterControllerPlugin, asset_plugin};

use crate::prelude::*;

mod player;
mod prelude;

fn main() {
    let mut app = App::new();

    // Bevy default plugins
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "The Curse".to_string(),
                    name: Some("thecurse".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(asset_plugin()),
    );

    // Ecosystem plugins
    app.add_plugins(PhysicsPlugins::default());

    // Custom plugins
    app.add_plugins((
        thecurse_core::debug::plugin,
        CameraControllerPlugin::<AppState>::default(),
        CharacterControllerPlugin,
        player::plugin,
    ));

    // States
    app.init_state::<AppState>();

    app.add_systems(Startup, demo_scene);

    app.run();
}

#[derive(States, Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
enum AppState {
    #[default]
    Game,
}

fn demo_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Ground"),
        Transform::default(),
        Visibility::Visible,
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.)))),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(Color::srgb(0.1, 0., 0.5)))),
    ));
}

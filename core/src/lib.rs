pub mod prelude;
use bevy::window::{CursorOptions, PrimaryWindow};
use prelude::*;

pub mod animation;
pub mod assets;
mod console;
pub mod creatures;
pub mod debug;
pub mod items;
mod navmesh;
pub mod networking;
mod shader_utils;
pub mod spells;
pub mod terrain;
pub mod utils;

mod shared;
pub use shared::*;

mod character_controller;
pub use character_controller::*;

mod camera_controller;
pub use camera_controller::*;

pub fn default_plugins<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        // Ecosystem plugins
        app.add_plugins((PhysicsPlugins::default(), PhysicsPickingPlugin));
        app.insert_resource(PhysicsPickingSettings {
            require_markers: true,
        });
        app.add_plugins(bevy_skein::SkeinPlugin::default());
        app.add_plugins((
            VleueNavigatorPlugin,
            NavmeshUpdaterPlugin::<Collider, Obstacle>::default(),
        ));

        // Custom plugins
        app.add_plugins((
            assets::plugin,
            utils::follow::FollowUtilPlugin::new(game_state),
            utils::billboard::plugin(game_state),
            shader_utils::plugin,
            animation::AnimationPlugin,
            navmesh::plugin(game_state),
            networking::client_plugin(game_state),
            console::plugin(game_state),
            creatures::plugin(game_state),
            character_controller::plugin(game_state),
            spells::plugin(game_state),
            terrain::plugin(game_state),
        ));

        app.add_systems(OnExit(game_state), (despawn_game_entities, show_cursor));
    }
}

pub fn asset_plugin() -> AssetPlugin {
    #[cfg(not(feature = "dev"))]
    return AssetPlugin::default();

    #[cfg(feature = "dev")]
    AssetPlugin {
        file_path: "../assets".to_string(),
        ..Default::default()
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
/// Marks an entity as to be despawned when the app leaves the game state
pub struct GameEntity;

fn despawn_game_entities(mut commands: Commands, query: Query<Entity, With<GameEntity>>) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}

fn show_cursor(mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    cursor.visible = true;
    cursor.grab_mode = bevy::window::CursorGrabMode::None;
}

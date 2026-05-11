pub mod prelude;
use prelude::*;

pub mod animation;
pub mod assets;
pub mod creatures;
pub mod debug;
pub mod items;
mod navmesh;
mod shader_utils;
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
            creatures::plugin(game_state),
            character_controller::plugin(game_state),
            terrain::plugin(game_state),
        ));
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

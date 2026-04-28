pub mod prelude;
use prelude::*;

pub mod assets;

pub mod utils;

pub mod debug;

mod shared;
pub use shared::*;

mod character_controller;
pub use character_controller::*;

mod camera_controller;
pub use camera_controller::*;

pub fn default_plugins<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_plugins((assets::plugin, character_controller::plugin(game_state)));
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

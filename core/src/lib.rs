pub mod prelude;

pub mod utils;

pub mod debug;

mod shared;
use bevy::asset::AssetPlugin;
pub use shared::*;

mod character_controller;
pub use character_controller::*;

mod camera_controller;
pub use camera_controller::*;

pub fn asset_plugin() -> AssetPlugin {
    #[cfg(not(feature = "dev"))]
    return AssetPlugin::default();

    #[cfg(feature = "dev")]
    AssetPlugin {
        file_path: "../assets".to_string(),
        ..Default::default()
    }
}

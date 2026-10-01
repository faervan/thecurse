use crate::prelude::*;

pub mod collision_layer;
pub mod environment;
pub mod helpers;
pub mod networking;
pub mod player;
pub mod prelude;

pub fn asset_plugin() -> AssetPlugin {
    AssetPlugin {
        file_path: "../assets".to_string(),
        ..Default::default()
    }
}

use crate::prelude::*;

pub mod networking;
pub mod prelude;

pub fn asset_plugin() -> AssetPlugin {
    AssetPlugin {
        file_path: "../assets".to_string(),
        ..Default::default()
    }
}

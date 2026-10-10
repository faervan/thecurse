use crate::prelude::*;

pub mod asset_loader;
pub mod gltf_loading;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(asset_loader::plugin);
}

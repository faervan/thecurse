use crate::prelude::*;

pub mod billboard;
pub mod follow;
pub mod gltf_instance_hooks;
pub mod gltf_loading;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        billboard::plugin,
        follow::FollowUtilPlugin::new(AppState::Game),
    ));
}

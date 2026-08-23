pub use thecurse_core::prelude::*;

pub use bevy::{
    light::{NotShadowCaster, NotShadowReceiver},
    render::render_resource::AsBindGroup,
};

pub use crate::AppState;
pub use crate::GameSettings;
pub use crate::scenes::GameScene;

pub use crate::utils::billboard::Billboarded;
pub use crate::utils::follow::Follow;
pub use crate::utils::gltf_instance_hooks::{
    ChildEntityPointer, GltfAnimationTarget, on_ready_insert_child_pointer,
};
pub use crate::utils::gltf_loading::GltfAnimationExtractionExt as _;
pub use crate::utils::gltf_loading::{GltfAssetPath, GltfLoadingHandle};

pub use crate::networking::tcp::ServerConnection;
pub use crate::networking::udp::Udp;

pub use crate::camera::{CameraController, CameraControllerAnchor};

pub use crate::hud::health_bar::ShowHealthBar;

pub use crate::weapon::{WeaponColliderHandle, WeaponSocketHandle};

pub use crate::player::cursor_target::CursorTargetPosition;

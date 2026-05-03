pub use std::f32::consts::{PI, TAU};
pub use std::marker::PhantomData;
pub use std::ops::{Deref, DerefMut};
pub use std::time::Duration;

pub use thiserror::Error;

pub use avian3d::prelude::*;
pub use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
pub use bevy::platform::collections::{HashMap, HashSet};
pub use bevy::prelude::*;

pub use crate::animation::AnimationExt as _;
pub use crate::assets::AssetResourceLoader as _;
pub use crate::shared::GameLayer;
pub use crate::utils::gltf_instance_hooks::{GltfAnimationTarget, on_ready_insert_child_pointer};
pub use crate::utils::gltf_loading::GltfAnimationExtractionExt as _;
pub use crate::utils::gltf_loading::{GltfAssetPath, GltfLoadingHandle};

pub use crate::creatures::goblin::SpawnGoblin;
pub use crate::creatures::health::{DealDamage, Health};
pub use crate::creatures::{Creature, CreatureBundle};

pub use crate::camera_controller::{CameraController, CameraControllerAnchor};
pub use crate::character_controller::actions::InterruptAction;
pub use crate::character_controller::{MainCharacter, Player, PlayerCharacterHandle, SpawnPlayer};

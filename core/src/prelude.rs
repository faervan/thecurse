pub use std::f32::consts::{PI, TAU};
pub use std::marker::PhantomData;
pub use std::ops::{Deref, DerefMut};
pub use std::time::Duration;

pub use thiserror::Error;

pub use avian3d::{math::*, prelude::*};
pub use bevy::ecs::{entity::EntityHashSet, lifecycle::HookContext, world::DeferredWorld};
pub use bevy::input::common_conditions::{input_just_pressed, input_just_released, input_pressed};
pub use bevy::platform::collections::{HashMap, HashSet};
pub use bevy::prelude::*;
pub use vleue_navigator::prelude::*;

pub use crate::animation::AnimationExt as _;
pub use crate::assets::AssetResourceLoader as _;
pub use crate::shared::GameLayer;
pub use crate::utils::billboard::Billboarded;
pub use crate::utils::gltf_instance_hooks::{GltfAnimationTarget, on_ready_insert_child_pointer};
pub use crate::utils::gltf_loading::GltfAnimationExtractionExt as _;
pub use crate::utils::gltf_loading::{GltfAssetPath, GltfLoadingHandle};

pub use crate::navmesh::Obstacle;

pub use crate::items::{Item, ItemEffect as _};

pub use crate::creatures::behavior::*;
pub use crate::creatures::goblin::{Goblin, SpawnGoblin};
pub use crate::creatures::health::*;
pub use crate::creatures::weapon::{WeaponColliderHandle, WeaponSocketHandle};
pub use crate::creatures::{Creature, CreatureBundle};

pub use crate::camera_controller::{CameraController, CameraControllerAnchor};
pub use crate::character_controller::actions::InterruptAction;
pub use crate::character_controller::cursor_target::CursorTargetPosition;
pub use crate::character_controller::{MainCharacter, Player, PlayerCharacterHandle, SpawnPlayer};

pub use crate::spells::SpawnSpellVoid;

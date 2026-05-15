pub use std::f32::consts::{PI, TAU};
pub use std::marker::PhantomData;
pub use std::ops::{Deref, DerefMut};
pub use std::time::Duration;

pub use thiserror::Error;

pub use serde::{Deserialize, Serialize};

pub use smol::channel::{Receiver, Sender};

pub use avian3d::{math::*, prelude::*};
pub use bevy::ecs::{entity::EntityHashSet, lifecycle::HookContext, world::DeferredWorld};
pub use bevy::input::common_conditions::{input_just_pressed, input_just_released, input_pressed};
pub use bevy::platform::collections::{HashMap, HashSet};
pub use bevy::prelude::*;
pub use vleue_navigator::prelude::*;

pub use crate::GameEntity;
pub use crate::animation::AnimationExt as _;
pub use crate::assets::AssetResourceLoader as _;
pub use crate::shared::GameLayer;

pub use crate::navmesh::Obstacle;

pub use crate::items::{Item, ItemEffect as _};

pub use crate::creatures::behavior::*;
pub use crate::creatures::crowd_control::*;
pub use crate::creatures::goblin::Goblin;
pub use crate::creatures::health::*;
pub use crate::creatures::player::{MainCharacter, Player};
pub use crate::creatures::{Creature, IsCreature};

pub use crate::spells::SpawnSpellVoid;

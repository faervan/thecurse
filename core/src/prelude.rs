pub use std::collections::{HashMap, VecDeque};
pub use std::fmt::Debug;
pub use std::marker::PhantomData;
pub use std::net::SocketAddr;
pub use std::time::{Duration, Instant};

pub use bevy::ecs::lifecycle::HookContext;
pub use bevy::ecs::world::DeferredWorld;
pub use bevy::prelude::*;

pub use avian3d::math::PI;
pub use avian3d::prelude::*;

pub use mini_udp::{
    Error as MiniUdpError,
    prelude::*,
    ring_buffer::{RingBuffer, wrapping_gt},
};

pub use clap::{self, Parser};

pub use crate::asset_plugin;
pub use crate::collision_layer::GameLayer;
pub use crate::networking::{
    ClientId, MsgToClient, MsgToServer, PROTOCOL_VERSION, SERVER_TIMESTEP,
};
pub use crate::player::{Player, PlayerAction, PlayerState};

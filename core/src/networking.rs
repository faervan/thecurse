use std::fmt::Display;

use crate::{environment::EnvironmentObj, prelude::*};

pub const PROTOCOL_VERSION: u32 = 0;

pub const SERVER_TIMESTEP: Duration = Duration::from_micros(15625);

#[derive(ByteRepr, Debug)]
pub enum MsgToServer {
    Ping { id: u16 },
    Action { id: u16, action: PlayerAction },
    Disconnect,
}

#[derive(ByteRepr, Debug, Clone)]
pub enum MsgToClient {
    ServerTick {
        server_tick_id: u16,
    },
    PingResponse {
        id: u16,
    },
    Connected {
        id: ClientId,
        state: PlayerState,
    },
    EnvironmentObj {
        kind: EnvironmentObj,
        translation: [f32; 3],
    },
    PlayerInfo {
        id: ClientId,
        state: PlayerState,
    },
    PlayerConnected {
        id: ClientId,
        state: PlayerState,
    },
    PlayerDisconnected {
        id: ClientId,
    },
    PlayerAction {
        id: ClientId,
        action_id: u16,
        action: PlayerAction,
    },
    StateUpdate {
        last_processed_action: u16,
        state: PlayerState,
    },
    PlayerStateUpdate {
        id: ClientId,
        state: PlayerState,
    },
}

#[derive(ByteRepr, Reflect, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ClientId(pub u64);

impl Display for ClientId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

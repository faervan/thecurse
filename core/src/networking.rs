use crate::prelude::*;

pub const PROTOCOL_VERSION: u32 = 0;
pub type UdpClientCfg = UdpContext<MsgToServer, MsgToClient, PROTOCOL_VERSION>;
pub type UdpServerCfg = <UdpClientCfg as MiniUdpContext>::Reverse;

pub const SERVER_TIMESTEP: Duration = Duration::from_micros(15625);

#[derive(ByteRepr, Debug)]
pub enum MsgToServer {
    Ping { id: u16 },
    Action { id: u16, action: PlayerAction },
    Disconnect,
}

#[derive(ByteRepr, Debug)]
pub enum MsgToClient {
    PingResponse {
        id: u16,
    },
    Connected {
        id: ClientId,
        state: PlayerState,
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
    ActionAck {
        last_processed_action: u16,
        state: PlayerState,
    },
}

#[derive(ByteRepr, Component, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ClientId(pub u64);

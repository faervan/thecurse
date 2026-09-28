use crate::prelude::*;

pub const PROTOCOL_VERSION: u32 = 0;
pub type UdpClientCfg = UdpContext<MsgToServer, MsgToClient, PROTOCOL_VERSION>;
pub type UdpServerCfg = <UdpClientCfg as MiniUdpContext>::Reverse;

pub const SERVER_TIMESTEP: Duration = Duration::from_micros(15625);

#[derive(ByteRepr, Debug)]
pub enum MsgToServer {
    Ping { id: u16 },
    Disconnect,
}

#[derive(ByteRepr, Debug)]
pub enum MsgToClient {
    PingResponse { id: u16 },
    Connected { id: ClientId, translation: [f32; 3] },
    PlayerInfo { id: ClientId, translation: [f32; 3] },
    PlayerConnected { id: ClientId, translation: [f32; 3] },
    PlayerDisconnected { id: ClientId },
}

#[derive(ByteRepr, Component, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ClientId(pub u64);

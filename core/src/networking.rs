use crate::prelude::*;

pub const PROTOCOL_VERSION: u32 = 0;
pub type UdpClientCfg = UdpContext<MsgToServer, MsgToClient, PROTOCOL_VERSION>;
pub type UdpServerCfg = <UdpClientCfg as MiniUdpContext>::Reverse;

pub const SERVER_TIMESTEP: Duration = Duration::from_millis(15625);

#[derive(ByteRepr, Debug)]
pub enum MsgToServer {
    Disconnect,
}

#[derive(ByteRepr, Debug)]
pub enum MsgToClient {
    Hello,
}

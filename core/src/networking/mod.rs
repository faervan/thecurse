use crate::prelude::*;
use io_util::TheCurseReadWriteExt;

mod handle_udp;
pub use handle_udp::{UdpMsgToClient, UdpMsgToServer};

pub mod io_util;

pub const PROTOCOL_VERSION: u32 = 0;
pub const SERVER_TIMESTEP: Duration = Duration::from_micros(15625);

#[derive(Debug, Serialize, Deserialize)]
pub enum TcpMsgToServer {
    Connect,
    Message(String),
    Disconnect,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TcpMsgToClient {
    ConnectionRefused,
    ConnectionAccepted { client_id: ClientId, world: String },
    ClientConnected(ClientId),
    ClientDisconnected(ClientId),
    Message { sender: ClientId, message: String },
}

impl TheCurseReadWriteExt for TcpMsgToServer {}
impl TheCurseReadWriteExt for TcpMsgToClient {}

#[derive(
    ByteRepr, Component, Debug, Serialize, Deserialize, Reflect, PartialEq, Eq, Hash, Clone, Copy,
)]
#[reflect(Component)]
pub struct ClientId(pub usize);

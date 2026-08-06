use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::prelude::*;

pub const UDP_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7188);

#[derive(ByteRepr, Debug, Clone)]
pub enum UdpMsgToServer {
    Connect(ClientId),
    Disconnect,
    Ping { id: u16 },
    Action { id: u16, action: PlayerAction },
}

#[derive(ByteRepr, Debug, Clone)]
pub enum UdpMsgToClient {
    Connected {
        translation: [f32; 3],
    },
    PlayerConnected {
        id: ClientId,
        translation: [f32; 3],
    },
    PlayerDisconnected {
        id: ClientId,
    },
    Ping {
        id: u16,
    },
    PlayerAction {
        client_id: ClientId,
        /// The id of the last [UdpMsgToServer] sent by the client that was processed by the server at
        /// the time this [UdpMsgToClient] was constructed.
        last_processed_action: u16,
        action: PlayerActionBroadcast,
    },
}

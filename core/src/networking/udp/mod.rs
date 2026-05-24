use std::net::{IpAddr, Ipv4Addr, SocketAddr};

mod bytes;
pub use bytes::*;

mod packet_sender;
pub use packet_sender::UdpCommunicator;

mod ring_buffer;

const UDP_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7188);

enum Message {
    X,
}

impl ByteRepr for Message {
    const MIN_LEN: usize = 1;
    const MAX_LEN: usize = 1;
    fn byte_len(&self) -> usize {
        match self {
            Self::X => 1,
        }
    }
    fn from_bytes(bytes: &[u8]) -> bevy::ecs::error::Result<Self, ByteReprError> {
        match bytes.first() {
            Some(1) => Ok(Self::X),
            _ => Err(ByteReprError::InvalidValue),
        }
    }
    fn write_as_bytes(&self, bytes: &mut [u8]) {
        match self {
            Self::X => bytes[0] = 1,
        }
    }
}

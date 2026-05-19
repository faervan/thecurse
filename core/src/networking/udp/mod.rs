use crate::networking::udp::bytes::{ByteRepr, ByteReprError};

mod bytes;
mod packet_sender;
mod ring_buffer;

enum Message {
    X,
}

impl ByteRepr for Message {
    const MIN_LEN: usize = 1;
    const MAX_LEN: usize = 1;
    fn len(&self) -> usize {
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

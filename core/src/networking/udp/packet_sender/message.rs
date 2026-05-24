use crate::networking::udp::bytes::{ByteRepr, ByteReprError};

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub(super) struct UdpMessage<M: ByteRepr> {
    pub(super) reliable: bool,
    pub(super) inner: M,
}

impl<M: ByteRepr> UdpMessage<M> {
    pub fn new(inner: M) -> Self {
        Self {
            reliable: true,
            inner,
        }
    }
}

impl<M: ByteRepr> ByteRepr for UdpMessage<M> {
    const MIN_LEN: usize = 1 + M::MIN_LEN;
    const MAX_LEN: usize = 1 + M::MAX_LEN;
    fn byte_len(&self) -> usize {
        1 + self.inner.byte_len()
    }
    fn write_as_bytes(&self, bytes: &mut [u8]) {
        bytes[0] = match self.reliable {
            true => 1,
            false => 0,
        };
        self.inner.write_as_bytes(&mut bytes[1..]);
    }
    fn from_bytes(
        bytes: &[u8],
    ) -> bevy::ecs::error::Result<Self, crate::networking::udp::bytes::ByteReprError> {
        Ok(Self {
            reliable: match bytes.first().ok_or(ByteReprError::InvalidValue)? {
                1 => true,
                0 => false,
                _ => return Err(ByteReprError::InvalidValue),
            },
            inner: M::from_bytes(&bytes[1..])?,
        })
    }
}

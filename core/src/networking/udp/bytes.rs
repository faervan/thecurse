use crate::networking::udp::packet_sender::InnerUdpMessage;
use crate::prelude::*;

pub trait StaticByteRepr<const LEN: usize> {
    fn as_bytes(&self) -> [u8; LEN];
    fn from_bytes(bytes: &[u8; LEN]) -> Self;
}

pub trait ByteRepr: Sized {
    const MIN_LEN: usize;
    const MAX_LEN: usize;
    fn len(&self) -> usize;
    /// *Panics* if `bytes.len() < self.len()`
    fn write_as_bytes(&self, bytes: &mut [u8]);
    fn from_bytes(bytes: &[u8]) -> Result<Self, ByteReprError>;

    /// Writes all items of `v` into `bytes`.
    /// Returns the number of items of `v` that fit into and have been written to `bytes`.
    fn write_many(v: &[Self], bytes: &mut [u8]) -> usize {
        let mut ptr = 0;
        for (i, v) in v.iter().enumerate() {
            if bytes.len() - ptr < v.len() {
                return i;
            }
            v.write_as_bytes(&mut bytes[ptr..]);
            ptr += v.len();
        }
        v.len()
    }
    fn read_many(bytes: &[u8]) -> Vec<Self> {
        let mut ptr = 0;
        let mut out = vec![];
        loop {
            match Self::from_bytes(&bytes[ptr..]) {
                Ok(v) => {
                    ptr += v.len();
                    out.push(v);
                }
                Err(e) => {
                    warn!("{e}");
                    return out;
                }
            }
        }
    }
}

impl ByteRepr for InnerUdpMessage {
    const MIN_LEN: usize = 1;
    const MAX_LEN: usize = 3;
    fn len(&self) -> usize {
        match self {
            Self::Hello => 1,
            Self::Wave(n) if n >> 8 == 0 => 2,
            Self::Wave(_) => 3,
        }
    }
    fn write_as_bytes(&self, bytes: &mut [u8]) {
        match self {
            Self::Hello => bytes[0] = 1,
            Self::Wave(n) if n >> 8 == 0 => {
                bytes[0] = 2;
                bytes[1] = *n as u8;
            }
            Self::Wave(n) => {
                bytes[0] = 3;
                bytes[1..3].copy_from_slice(&n.to_le_bytes());
            }
        }
    }
    fn from_bytes(bytes: &[u8]) -> Result<Self, ByteReprError> {
        match bytes.first().ok_or(ByteReprError::InvalidValue)? {
            1 => Ok(Self::Hello),
            2 => match bytes.get(1) {
                Some(n) => Ok(Self::Wave(*n as u16)),
                None => Err(ByteReprError::InvalidValue),
            },
            3 => {
                if let Ok(bytes) = bytes[1..3].try_into() {
                    Ok(Self::Wave(u16::from_le_bytes(bytes)))
                } else {
                    Err(ByteReprError::InvalidValue)
                }
            }
            _ => Err(ByteReprError::InvalidValue),
        }
    }
}

#[derive(Debug, Error)]
pub enum ByteReprError {
    #[error("Invalid value")]
    InvalidValue,
}

#[cfg(test)]
mod test {
    use crate::networking::udp::{bytes::ByteRepr as _, packet_sender::InnerUdpMessage};

    #[test]
    fn byte_repr() {
        let hello = InnerUdpMessage::Hello;
        let w5 = InnerUdpMessage::Wave(5);
        let w_max = InnerUdpMessage::Wave(u16::MAX);
        let v = vec![hello, w5, w_max];
        assert_eq!(v.len(), 3);
        let mut bytes = [0; InnerUdpMessage::MAX_LEN * 3];
        InnerUdpMessage::write_many(&v, &mut bytes);

        println!("bytes: {bytes:?}");
        let messages = InnerUdpMessage::read_many(&bytes);
        assert_eq!(messages, v);

        let mut bytes = vec![0; hello.len() + w5.len()];
        assert_eq!(InnerUdpMessage::write_many(&v, &mut bytes), 2);
        assert_eq!(InnerUdpMessage::read_many(&bytes), vec![hello, w5]);
    }
}

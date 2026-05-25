use crate::networking::udp::{
    bytes::{ByteRepr, ByteReprError, StaticByteRepr as _},
    packet_sender::packet_ack::PacketAck,
};

/// The maximum allowed length of the data part of a UDP packet.
/// The total maximum length is computed by adding the header length as well.
pub(super) const MAX_PACKET_DATA_LEN: usize = 1024;
/// 4 bytes for the CRC, then the [`PacketAck`], then 1 byte extra metadata (reliable, ordered)
pub(super) const PACKET_HEADER_LEN: usize = 4 + PacketAck::LEN + 1;
/// The maximum allowed length of a UDP packet.
pub(super) const MAX_PACKET_LEN: usize = PACKET_HEADER_LEN + MAX_PACKET_DATA_LEN;

const PROTOCOL_VERSION: u32 = 0x00_00_00_01;
/// [`crc::CRC_32_BZIP2`] with `init` set to [`PROTOCOL_VERSION`]
const CRC_ALGORITHM: crc::Algorithm<u32> = crc::Algorithm {
    width: 32,
    poly: 0x04c11db7,
    init: PROTOCOL_VERSION,
    refin: false,
    refout: false,
    xorout: 0xffffffff,
    check: 0xfc891918,
    residue: 0xc704dd7b,
};
const CRC: crc::Crc<u32> = crc::Crc::<u32>::new(&CRC_ALGORITHM);

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Packet<M: ByteRepr> {
    pub(super) ack: PacketAck,
    /// TODO!
    pub(super) reliable: bool,
    /// TODO!
    pub(super) ordered: bool,
    /// If `messages.is_empty()`, then this was send as a heartbeat packet
    pub(super) messages: Vec<M>,
}

impl<M: ByteRepr> Packet<M> {
    #[inline(always)]
    pub fn new(ack: PacketAck, messages: impl IntoIterator<Item = M>) -> Self {
        Self {
            ack,
            reliable: true,
            ordered: false,
            messages: messages.into_iter().collect(),
        }
    }

    #[inline(always)]
    pub fn heartbeat(ack: PacketAck) -> Self {
        Self {
            ack,
            reliable: false,
            ordered: false,
            messages: vec![],
        }
    }
}

impl<M: ByteRepr> ByteRepr for Packet<M> {
    const MIN_LEN: usize = PACKET_HEADER_LEN + M::MIN_LEN;
    const MAX_LEN: usize = MAX_PACKET_LEN;
    fn byte_len(&self) -> usize {
        PACKET_HEADER_LEN
            + self.messages.iter().fold(0, |mut acc, m| {
                acc += m.byte_len();
                acc
            })
    }
    fn write_as_bytes(&self, bytes: &mut [u8]) {
        bytes[4..4 + PacketAck::LEN].copy_from_slice(&self.ack.as_bytes());
        let reliable = (self.reliable as u8) << 0;
        let ordered = (self.ordered as u8) << 1;
        bytes[4 + PacketAck::LEN] = reliable | ordered;
        let (_items_written, bytes_written) =
            M::write_many(&self.messages, &mut bytes[PACKET_HEADER_LEN..]);
        let crc = CRC
            .checksum(&bytes[4..PACKET_HEADER_LEN + bytes_written])
            .to_le_bytes();
        bytes[..4].copy_from_slice(&crc);
    }
    fn from_bytes(bytes: &[u8]) -> bevy::ecs::error::Result<Self, ByteReprError> {
        let (messages, body_len) = M::read_many(&bytes[PACKET_HEADER_LEN..]);

        let crc = u32::from_le_bytes(bytes[..4].try_into()?);
        if CRC.checksum(&bytes[4..PACKET_HEADER_LEN + body_len]) != crc {
            return Err(ByteReprError::CrcMismatch);
        }

        let meta_byte = bytes
            .get(4 + PacketAck::LEN)
            .ok_or(ByteReprError::InvalidValue)?;
        let reliable = meta_byte & 1 << 0 != 0;
        let ordered = meta_byte & 1 << 1 != 0;

        Ok(Self {
            ack: PacketAck::from_bytes(bytes[4..4 + PacketAck::LEN].try_into()?),
            reliable,
            ordered,
            messages,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::networking::udp::{
        bytes::ByteRepr,
        packet_sender::{InnerUdpMessage, packet::Packet},
    };
    use crate::prelude::*;

    #[test]
    fn packet_byte_repr() {
        let com = UdpCommunicator::<InnerUdpMessage>::default();
        let packet = Packet::new(
            com.create_ack(0),
            [
                InnerUdpMessage::Wave(12),
                InnerUdpMessage::Wave(9284),
                InnerUdpMessage::Hello,
            ],
        );
        let mut buf = [0; Packet::<InnerUdpMessage>::MAX_LEN];
        packet.write_as_bytes(&mut buf);
        assert_eq!(Packet::<InnerUdpMessage>::from_bytes(&buf).unwrap(), packet);
    }
}

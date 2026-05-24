use crate::networking::udp::{
    bytes::{ByteRepr, ByteReprError, StaticByteRepr as _},
    packet_sender::{UdpMessage, packet_ack::PacketAck},
};

/// The maximum allowed length of the data part of a UDP packet.
/// The total maximum length is computed by adding the header length as well.
pub(super) const MAX_PACKET_DATA_LEN: usize = 1024;
pub(super) const PACKET_HEADER_LEN: usize = 4 + PacketAck::LEN;
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
    pub(super) messages: Vec<UdpMessage<M>>,
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
        let (_items_written, bytes_written) =
            UdpMessage::<M>::write_many(&self.messages, &mut bytes[PACKET_HEADER_LEN..]);
        let crc = CRC
            .checksum(&bytes[4..PACKET_HEADER_LEN + bytes_written])
            .to_le_bytes();
        bytes[..4].copy_from_slice(&crc);
    }
    fn from_bytes(bytes: &[u8]) -> bevy::ecs::error::Result<Self, ByteReprError> {
        let (messages, body_len) = UdpMessage::<M>::read_many(&bytes[PACKET_HEADER_LEN..]);

        let crc = u32::from_le_bytes(bytes[..4].try_into()?);
        if CRC.checksum(&bytes[4..PACKET_HEADER_LEN + body_len]) != crc {
            return Err(ByteReprError::CrcMismatch);
        }

        Ok(Self {
            ack: PacketAck::from_bytes(bytes[4..4 + PacketAck::LEN].try_into()?),
            messages,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::networking::udp::{
        bytes::ByteRepr,
        packet_sender::{InnerUdpMessage, UdpMessage, packet::Packet},
    };
    use crate::prelude::*;

    #[test]
    fn packet_byte_repr() {
        let com = UdpCommunicator::<InnerUdpMessage>::default();
        let packet = Packet {
            ack: com.create_ack(0),
            messages: [
                InnerUdpMessage::Wave(12),
                InnerUdpMessage::Wave(9284),
                InnerUdpMessage::Hello,
            ]
            .into_iter()
            .map(UdpMessage::new)
            .collect(),
        };
        let mut buf = [0; Packet::<InnerUdpMessage>::MAX_LEN];
        packet.write_as_bytes(&mut buf);
        assert_eq!(Packet::<InnerUdpMessage>::from_bytes(&buf).unwrap(), packet);
    }
}

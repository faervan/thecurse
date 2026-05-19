use crate::networking::udp::{
    bytes::{ByteRepr, StaticByteRepr},
    packet_sender::UdpCommunicator,
};

struct PacketAck {
    newest_send: u16,
    newest_received: u16,
    ack_bits: u32,
}

impl StaticByteRepr<8> for PacketAck {
    fn as_bytes(&self) -> [u8; 8] {
        let mut bytes = [0; 8];
        bytes[..2].copy_from_slice(&self.newest_send.to_le_bytes());
        bytes[2..4].copy_from_slice(&self.newest_received.to_le_bytes());
        bytes[4..].copy_from_slice(&self.ack_bits.to_le_bytes());
        bytes
    }
    fn from_bytes(bytes: &[u8; 8]) -> Self {
        Self {
            newest_send: u16::from_le_bytes(bytes[0..2].try_into().unwrap()),
            newest_received: u16::from_le_bytes(bytes[2..4].try_into().unwrap()),
            ack_bits: u32::from_le_bytes(bytes[4..].try_into().unwrap()),
        }
    }
}

impl<M: ByteRepr> UdpCommunicator<M> {
    fn acknowledge(&mut self, ack: PacketAck) {
        for i in 0..32 {
            if ack.ack_bits & 1 << i != 0 {
                let index = ack.newest_received.wrapping_sub(i);
                self.received_packets.take(index);
            }
        }
    }

    fn create_ack(&self) -> PacketAck {
        let mut ack_bits = 0;
        let newest_received = self.received_packets.get_newest_index();
        for i in self.received_packets.keys() {
            ack_bits |= 1 << newest_received.wrapping_sub(i) as u32;
        }
        PacketAck {
            newest_send: self.send_packets.get_newest_index(),
            newest_received,
            ack_bits,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::networking::udp::Message;

    #[test]
    fn acknowledge() {
        let mut com = super::UdpCommunicator::<Message>::new();
        // Those two are overridden immediately
        com.received_packets.push(());
        com.received_packets.insert((), 3);
        //
        com.received_packets.insert((), u16::MAX - 30);
        com.received_packets.insert((), u16::MAX);
        com.received_packets.push(());
        let ack = com.create_ack();
        assert_eq!(com.received_packets.iter().count(), 3);
        com.acknowledge(ack);
        assert_eq!(com.received_packets.iter().count(), 0);
    }
}

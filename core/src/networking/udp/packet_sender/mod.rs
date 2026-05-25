use std::{
    collections::VecDeque,
    fmt::Debug,
    net::{ToSocketAddrs, UdpSocket},
    time::Instant,
};

use crate::networking::udp::{packet_sender::packet::MAX_PACKET_DATA_LEN, ring_buffer::RingBuffer};
use crate::{
    networking::udp::{
        bytes::ByteRepr,
        packet_sender::packet::{MAX_PACKET_LEN, Packet},
    },
    prelude::*,
};

mod packet;
mod packet_ack;

#[derive(Debug, PartialEq, Hash, Eq, Clone, Copy)]
pub enum InnerUdpMessage {
    Hello,
    Wave(u16),
}

pub struct UdpCommunicator<M: ByteRepr> {
    socket: UdpSocket,
    reliable_send_packets: RingBuffer<(Instant, Packet<M>)>,
    unreliable_send_packet_id: u16,
    unreliable_send_packets: VecDeque<Packet<M>>,
    received_packets: RingBuffer<()>,
    msg_send_queue: VecDeque<M>,
    msg_recv_queue: VecDeque<M>,
    data_buffer: [u8; MAX_PACKET_LEN],
    /// If this is `true`, a packet has been received more than once, potentially meaning that we
    /// have to send an ack to the other side.
    received_packet_duplicate: bool,
    #[cfg(test)]
    fake_unreliable: bool,
    #[cfg(test)]
    debug_logs: bool,
}

impl<M: ByteRepr> Default for UdpCommunicator<M> {
    fn default() -> Self {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind to udp socket");
        socket
            .set_nonblocking(true)
            .expect("Failed to set udp socket to nonblocking mode");
        Self {
            socket,
            reliable_send_packets: RingBuffer::new(),
            unreliable_send_packet_id: 0,
            unreliable_send_packets: VecDeque::new(),
            received_packets: RingBuffer::new(),
            msg_send_queue: VecDeque::new(),
            msg_recv_queue: VecDeque::new(),
            data_buffer: [0; MAX_PACKET_LEN],
            received_packet_duplicate: false,
            #[cfg(test)]
            fake_unreliable: false,
            #[cfg(test)]
            debug_logs: false,
        }
    }
}

impl<M: ByteRepr> UdpCommunicator<M> {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> Self {
        let socket = UdpSocket::bind(addr).expect("Failed to bind to udp socket");
        socket
            .set_nonblocking(true)
            .expect("Failed to set udp socket to nonblocking mode");
        Self {
            socket,
            reliable_send_packets: RingBuffer::new(),
            unreliable_send_packet_id: 0,
            unreliable_send_packets: VecDeque::new(),
            received_packets: RingBuffer::new(),
            msg_send_queue: VecDeque::new(),
            msg_recv_queue: VecDeque::new(),
            data_buffer: [0; MAX_PACKET_LEN],
            received_packet_duplicate: false,
            #[cfg(test)]
            fake_unreliable: false,
            #[cfg(test)]
            debug_logs: false,
        }
    }

    #[cfg(test)]
    fn with_fake_unreliablity(mut self) -> Self {
        self.fake_unreliable = true;
        self
    }

    #[cfg(test)]
    fn with_debug_logs(mut self) -> Self {
        self.debug_logs = true;
        self
    }

    #[inline]
    pub fn connect<A: ToSocketAddrs>(&self, addr: A) -> Result<(), std::io::Error> {
        self.socket.connect(addr)
    }

    #[inline(always)]
    pub fn write(&mut self, message: M) {
        self.msg_send_queue.push_back(message);
    }

    #[inline(always)]
    pub fn read(&mut self) -> Option<M> {
        self.msg_recv_queue.pop_front()
    }

    #[inline(always)]
    /// TODO! Remove where Debug
    pub fn tick(&mut self)
    where
        M: Debug,
    {
        self.receive();
        if self.received_packet_duplicate && self.msg_send_queue.is_empty() {
            self.received_packet_duplicate = false;
            let sequence_id = self.unreliable_send_packet_id;
            self.unreliable_send_packet_id = self.unreliable_send_packet_id.wrapping_add(1);
            let packet = Packet::heartbeat(self.create_ack(sequence_id));
            #[cfg(test)]
            if self.debug_logs {
                debug!("Constructed new hearbeat packet #{sequence_id}");
            }
            self.unreliable_send_packets.push_back(packet);
        }
        self.flush_messages();
        self.send_packets();
    }

    /// TODO! Remove where Debug
    pub fn receive(&mut self)
    where
        M: Debug,
    {
        while let Ok(n) = self.socket.recv(&mut self.data_buffer) {
            match Packet::<M>::from_bytes(&self.data_buffer[..n]) {
                Ok(packet) => {
                    #[cfg(test)]
                    // Fake UDP unreliability
                    if self.fake_unreliable && rand::random_bool(0.5) {
                        continue;
                    }
                    // Heartbeat
                    if packet.messages.is_empty() {
                        #[cfg(test)]
                        debug!("Received heartbeat packet #{}", packet.ack.sequence_id);
                        self.acknowledge(packet.ack);
                        continue;
                    }

                    if self.received_packets.get(packet.ack.sequence_id).is_some() {
                        self.received_packet_duplicate = true;
                        #[cfg(test)]
                        debug!("Received duplicate packet #{}", packet.ack.sequence_id);
                        continue;
                    }
                    if super::ring_buffer::wrapping_gt(
                        self.received_packets.get_newest_index().wrapping_sub(31),
                        packet.ack.sequence_id,
                        64,
                    ) {
                        self.received_packet_duplicate = true;
                        #[cfg(test)]
                        debug!("Received too old packet #{}", packet.ack.sequence_id);
                        continue;
                    }
                    self.received_packets.insert(packet.ack.sequence_id, ());
                    self.msg_recv_queue.extend(packet.messages);
                    self.acknowledge(packet.ack);
                }
                Err(e) => warn!("Received invalid packet: {e}"),
            }
        }
    }

    /// TODO! Remove where Debug
    pub fn flush_messages(&mut self)
    where
        M: Debug,
    {
        while !self.reliable_send_packets.push_will_override() && !self.msg_send_queue.is_empty() {
            let mut available_bytes = MAX_PACKET_DATA_LEN;
            let mut included_msgs = 0;
            for msg in self.msg_send_queue.iter() {
                if msg.byte_len() <= available_bytes {
                    available_bytes -= msg.byte_len();
                    included_msgs += 1;
                } else {
                    // TODO! Maybe include other messages here that are small enough, but that
                    // would make message ordering arbitrary
                    break;
                }
            }
            if included_msgs == 0 {
                error!(
                    "Msg {:#?} is too large to fit {} bytes, but the max packet size is {}",
                    self.msg_send_queue[0],
                    self.msg_send_queue[0].byte_len(),
                    MAX_PACKET_DATA_LEN
                );
            }
            let sequence_id = self.reliable_send_packets.get_next_index();
            let packet = Packet {
                ack: self.create_ack(sequence_id),
                reliable: true,
                ordered: true,
                messages: self.msg_send_queue.drain(..included_msgs).collect(),
            };
            #[cfg(test)]
            if self.debug_logs {
                debug!("Constructed new packet #{sequence_id} with {included_msgs} messages");
            }
            self.reliable_send_packets
                .push((Instant::now() - Duration::from_secs(1), packet));
        }
    }

    fn send_packets(&mut self) {
        for (last_send, packet) in self.reliable_send_packets.iter_mut() {
            let send_cooldown = if cfg!(test) {
                Duration::from_millis(3)
            } else {
                Duration::from_millis(100)
            };
            if last_send.elapsed() > send_cooldown {
                *last_send = Instant::now();
                packet.write_as_bytes(&mut self.data_buffer);
                if let Err(e) = self.socket.send(&self.data_buffer[..packet.byte_len()]) {
                    error!("Failed to send packet: {e}");
                }
            }
        }
        for packet in self.unreliable_send_packets.drain(..) {
            packet.write_as_bytes(&mut self.data_buffer);
            if let Err(e) = self.socket.send(&self.data_buffer[..packet.byte_len()]) {
                error!("Failed to send packet: {e}");
            }
        }
    }

    #[inline(always)]
    pub fn has_work(&self) -> bool {
        !self.reliable_send_packets.is_empty()
    }
}

#[cfg(test)]
fn test_init<M>(port_offset: u16) -> (UdpCommunicator<M>, UdpCommunicator<M>)
where
    M: ByteRepr,
{
    let _ = bevy::log::tracing_subscriber::FmtSubscriber::builder()
        .with_test_writer()
        .with_max_level(bevy::log::Level::DEBUG)
        .try_init();
    let localhost = std::net::Ipv4Addr::new(127, 0, 0, 1);
    let localhost = std::net::IpAddr::V4(localhost);
    let addr1 = std::net::SocketAddr::new(localhost, port_offset);
    let addr2 = std::net::SocketAddr::new(localhost, port_offset + 1);
    let com1 = UdpCommunicator::<M>::bind(addr1);
    let com2 = UdpCommunicator::<M>::bind(addr2);
    assert!(com2.connect(addr1).is_ok());
    assert!(com1.connect(addr2).is_ok());
    (com1, com2)
}

#[cfg(test)]
mod test {
    use crate::networking::udp::packet_sender::InnerUdpMessage;
    use crate::prelude::*;

    #[test]
    fn packet_roundtrip() {
        let (mut com1, mut com2) = super::test_init::<InnerUdpMessage>(7200);
        let m1 = InnerUdpMessage::Hello;
        let m2 = InnerUdpMessage::Wave(1394);
        com2.write(m1);
        com2.write(m2);
        com2.tick();
        com1.tick();
        assert_eq!(com1.read(), Some(m1));
        assert_eq!(com1.read(), Some(m2));
        assert_eq!(com1.read(), None);
    }

    #[test]
    fn send_until_ack() {
        let (mut com1, mut com2) = super::test_init::<InnerUdpMessage>(7202);
        let m1 = InnerUdpMessage::Hello;
        com2.write(m1);
        com2.tick();

        let mut i = 0;
        while com2.has_work() {
            i += 1;
            com1.tick();
            if let Some(message) = com1.read() {
                assert_eq!(message, m1);
                debug!("com1 received: {message:?}");
                // Send a dummy packet back to acknowledge the received one
                com1.write(InnerUdpMessage::Wave(1));
            }
            com2.tick();
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(i, 2);
    }

    #[test]
    fn test_reliability() {
        let (mut com1, mut com2) = super::test_init::<InnerUdpMessage>(7204);
        com1 = com1.with_fake_unreliablity().with_debug_logs();
        com2 = com2.with_fake_unreliablity();
        let mut send = HashSet::new();
        assert!(send.insert(InnerUdpMessage::Hello));
        for i in 0..20000 {
            assert!(send.insert(InnerUdpMessage::Wave(i)));
        }
        for m in &send {
            com1.write(*m);
        }
        com1.tick();

        let mut received = HashSet::new();
        while com1.has_work() {
            com2.tick();
            while let Some(message) = com2.read() {
                assert!(received.insert(message));
            }
            com1.tick();
            std::thread::sleep(Duration::from_millis(1));
        }

        assert_eq!(received, send);
    }
}

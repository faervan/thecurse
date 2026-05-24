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
        packet_sender::{
            message::UdpMessage,
            packet::{MAX_PACKET_LEN, Packet},
        },
    },
    prelude::*,
};

mod message;
mod packet;
mod packet_ack;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InnerUdpMessage {
    Hello,
    Wave(u16),
}

pub struct UdpCommunicator<M: ByteRepr> {
    socket: UdpSocket,
    send_packets: RingBuffer<(Instant, Packet<M>)>,
    received_packets: RingBuffer<()>,
    msg_send_queue: VecDeque<UdpMessage<M>>,
    msg_recv_queue: VecDeque<M>,
    data_buffer: [u8; MAX_PACKET_LEN],
}

impl<M: ByteRepr> Default for UdpCommunicator<M> {
    fn default() -> Self {
        let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind to udp socket");
        socket
            .set_nonblocking(true)
            .expect("Failed to set udp socket to nonblocking mode");
        Self {
            socket,
            send_packets: RingBuffer::new(),
            received_packets: RingBuffer::new(),
            msg_send_queue: VecDeque::new(),
            msg_recv_queue: VecDeque::new(),
            data_buffer: [0; MAX_PACKET_LEN],
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
            send_packets: RingBuffer::new(),
            received_packets: RingBuffer::new(),
            msg_send_queue: VecDeque::new(),
            msg_recv_queue: VecDeque::new(),
            data_buffer: [0; MAX_PACKET_LEN],
        }
    }

    #[inline]
    pub fn connect<A: ToSocketAddrs>(&self, addr: A) -> Result<(), std::io::Error> {
        self.socket.connect(addr)
    }

    #[inline(always)]
    pub fn write(&mut self, message: M) {
        self.msg_send_queue.push_back(UdpMessage {
            reliable: true,
            inner: message,
        });
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
                    debug!("received packet: {packet:#?}");
                    self.acknowledge(packet.ack);
                    self.msg_recv_queue
                        .extend(packet.messages.into_iter().map(|m| m.inner));
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
        while !self.send_packets.insert_will_override() && !self.msg_send_queue.is_empty() {
            let mut available_bytes = MAX_PACKET_DATA_LEN;
            let mut included_msgs = 0;
            for msg in self.msg_send_queue.iter() {
                if msg.byte_len() <= available_bytes {
                    available_bytes -= msg.byte_len();
                    included_msgs += 1;
                } else {
                    // TODO! maybe include other messages here that are small enough, but that
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
            let sequence_id = self.send_packets.get_next_index();
            let packet = Packet {
                ack: self.create_ack(sequence_id),
                messages: self.msg_send_queue.drain(..included_msgs).collect(),
            };
            self.send_packets
                .push((Instant::now() - Duration::from_secs(1), packet));
        }
    }

    fn send_packets(&mut self) {
        for (last_send, packet) in self.send_packets.iter_mut() {
            if last_send.elapsed() > Duration::from_millis(100) {
                *last_send = Instant::now();
                packet.write_as_bytes(&mut self.data_buffer);
                if let Err(e) = self.socket.send(&self.data_buffer[..packet.byte_len()]) {
                    error!("Failed to send packet: {e}");
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::networking::udp::packet_sender::InnerUdpMessage;
    use crate::prelude::*;

    #[test]
    fn packet_roundtrip() {
        bevy::log::tracing_subscriber::FmtSubscriber::builder()
            .with_test_writer()
            .with_max_level(bevy::log::Level::DEBUG)
            .init();
        let addr1 = "127.0.0.1:7200";
        let mut com1 = UdpCommunicator::<InnerUdpMessage>::bind(addr1);
        let addr2 = "127.0.0.1:7201";
        let mut com2 = UdpCommunicator::<InnerUdpMessage>::bind(addr2);
        assert!(com2.connect(addr1).is_ok());
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
}

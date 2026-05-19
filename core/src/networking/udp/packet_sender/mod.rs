use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, ToSocketAddrs, UdpSocket},
    sync::Arc,
    time::Instant,
};

use crate::{networking::udp::bytes::ByteRepr, prelude::*};
use crate::{networking::udp::ring_buffer::RingBuffer, prelude::*};

mod packet_ack;

#[derive(Debug)]
pub struct Packet<M: ByteRepr> {
    messages: Vec<Arc<UdpMessage<M>>>,
}

#[derive(Debug)]
struct UdpMessage<M: ByteRepr> {
    reliable: bool,
    inner: M,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InnerUdpMessage {
    Hello,
    Wave(u16),
}

pub struct UdpCommunicator<M: ByteRepr> {
    socket: UdpSocket,
    last_send: Instant,
    send_packets: RingBuffer<Packet<M>>,
    received_packets: RingBuffer<()>,
    msg_send_queue: Vec<Arc<UdpMessage<M>>>,
    data_buffer: [u8; 1024],
}

const UDP_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7188);
impl<M: ByteRepr> Default for UdpCommunicator<M> {
    fn default() -> Self {
        Self {
            socket: UdpSocket::bind(UDP_ADDR).expect("Failed to bind to udp socket"),
            last_send: Instant::now(),
            send_packets: RingBuffer::new(),
            received_packets: RingBuffer::new(),
            msg_send_queue: vec![],
            data_buffer: [0; 1024],
        }
    }
}

impl<M: ByteRepr> UdpCommunicator<M> {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn connect<A: ToSocketAddrs>(&self, addr: A) -> Result<(), std::io::Error> {
        self.socket.connect(addr)
    }

    pub fn send(&mut self, message: M) {
        self.msg_send_queue.push(Arc::new(UdpMessage {
            reliable: true,
            inner: message,
        }));
    }

    pub fn tick(&mut self) {
        self.receive();
    }

    pub fn receive(&mut self) {
        while let Ok(n) = self.socket.recv(&mut self.data_buffer) {}
    }

    pub fn flush_messages(&mut self) {}
}

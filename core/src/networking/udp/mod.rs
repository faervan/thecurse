use crate::prelude::*;

mod ring_buffer;

pub struct Packet {
    messages: Vec<UdpMessage>,
}

pub enum UdpMessage {}

pub struct PacketSender;

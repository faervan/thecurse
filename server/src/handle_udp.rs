use std::net::{IpAddr, Ipv4Addr, UdpSocket};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup);

    app.add_systems(FixedUpdate, read_udp);
}

const UDP_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7188);
const MAX_PACKET_LENGTH: usize = 1024;

#[derive(Resource, Deref)]
struct Udp(UdpSocket);

fn setup(mut commands: Commands) {
    let socket = UdpSocket::bind(UDP_ADDR).unwrap();
    socket.set_nonblocking(true).unwrap();
    commands.insert_resource(Udp(socket));
}

#[derive(Deref, DerefMut)]
struct ReadBuffer([u8; MAX_PACKET_LENGTH]);
impl Default for ReadBuffer {
    fn default() -> Self {
        Self([0; MAX_PACKET_LENGTH])
    }
}

fn read_udp(udp: Res<Udp>, mut buf: Local<ReadBuffer>) {
    while let Ok((len, addr)) = udp.recv_from(&mut **buf) {
        if rand::random_ratio(2, 100) {
            // Drop packet, fake unreliability
            debug!("Packet of len {len} bytes was fake-dropped");
            continue;
        }
        debug!("Received {len} bytes from {addr:?} via UDP");
    }
}

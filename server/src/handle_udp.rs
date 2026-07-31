use std::net::{IpAddr, Ipv4Addr};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup);

    app.add_systems(FixedUpdate, read_udp);
}

const UDP_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7188);

#[derive(Resource, Deref, DerefMut)]
struct Udp(MultiUdpCommunicator<UdpMessage>);

fn setup(mut commands: Commands) {
    let com = MultiUdpCommunicator::bind(UDP_ADDR)
        .with_fake_unreliablity()
        .with_debug_logs();
    commands.insert_resource(Udp(com));
}

fn read_udp(mut udp: ResMut<Udp>) {
    udp.recv(|addr, mut com| {
        while let Some(msg) = com.read() {
            debug!("Received msg {msg:?} from {addr:?} via UDP");
        }
    });
}

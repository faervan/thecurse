use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

use crate::prelude::*;

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(OnEnter(game_state), setup);

        app.add_systems(
            Update,
            send_udp_packet.run_if(in_state(game_state).and(input_just_pressed(KeyCode::KeyU))),
        );
    }
}

const UDP_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7188);
const MAX_PACKET_LENGTH: usize = 1024;

#[derive(Resource, Deref)]
struct Udp(UdpSocket);

fn setup(mut commands: Commands) {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket.set_nonblocking(true).unwrap();
    socket.connect(UDP_ADDR).unwrap();
    commands.insert_resource(Udp(socket));
}

fn send_udp_packet(udp: Res<Udp>) {
    let n = rand::random_range(1..MAX_PACKET_LENGTH);
    udp.send(&vec![0; n]).unwrap();
}

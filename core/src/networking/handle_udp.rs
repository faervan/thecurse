use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::prelude::*;

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(OnEnter(game_state), setup);

        app.add_systems(Update, send_udp_packet.run_if(in_state(game_state)));
    }
}

const UDP_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7188);

#[derive(ByteRepr, Debug)]
pub enum UdpMessage {
    Hello(Vec<bool>),
}

#[derive(Resource, Deref, DerefMut)]
struct Udp(UdpCommunicator<UdpMessage>);

fn setup(mut commands: Commands) {
    let mut com = UdpCommunicator::default();
    com.connect(UDP_ADDR).unwrap();
    commands.insert_resource(Udp(com));
}

fn send_udp_packet(mut udp: ResMut<Udp>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::KeyU) {
        udp.write(UdpMessage::Hello(vec![false, true]));
    }
    udp.tick().unwrap();
}

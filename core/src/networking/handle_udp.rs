use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use crate::{
    creatures::player::{AttackState, AttackType},
    networking::ServerConnection,
    prelude::*,
};

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(OnEnter(game_state), setup);

        app.add_systems(
            Update,
            (
                send_udp_packet
                    .before(read_udp_messages)
                    .run_if(input_just_pressed(KeyCode::KeyU)),
                read_udp_messages,
            )
                .run_if(in_state(game_state)),
        );
    }
}

pub const UDP_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7188);

#[derive(ByteRepr, Debug)]
pub enum UdpMsgToServer {
    Connect(ClientId),
    Hello(Vec<bool>),
    PlayerAttack { ty: AttackType },
}

#[derive(ByteRepr, Debug, Clone)]
pub enum UdpMsgToClient {
    Connected,
    Hello(Vec<bool>),
    PlayerConnected { id: ClientId },
    PlayerAttack { id: ClientId, ty: AttackType },
}

#[derive(Resource, Deref, DerefMut)]
pub struct Udp {
    com: UdpCommunicator<UdpMsgToServer, UdpMsgToClient>,
}

fn setup(mut commands: Commands) {
    let mut com = UdpCommunicator::default();
    com.connect(UDP_ADDR).unwrap();
    commands.insert_resource(Udp { com });
}

fn send_udp_packet(mut udp: ResMut<Udp>) {
    udp.write(UdpMsgToServer::Hello(vec![false, true]));
}

fn read_udp_messages(mut udp: ResMut<Udp>, con: Res<ServerConnection>, mut commands: Commands) {
    udp.tick().unwrap();
    while let Some(msg) = udp.read() {
        debug!("Received msg via UDP: {msg:?}");
        match msg {
            UdpMsgToClient::Connected => {
                let Some(client_id) = con.client_id else {
                    error!("ClientId not initialized, not spawning player");
                    continue;
                };
                commands.spawn((MainCharacter, client_id));
            }
            UdpMsgToClient::Hello(_) => {}
            UdpMsgToClient::PlayerConnected { id } => {
                commands.spawn((Player, id));
            }
            UdpMsgToClient::PlayerAttack { id, ty } => {
                if let Some(entity) = con.clients.get(&id) {
                    commands.entity(*entity).insert(AttackState::Attacking {
                        timer: Timer::new(ty.duration(), TimerMode::Once),
                        ty,
                    });
                }
            }
        }
    }
}

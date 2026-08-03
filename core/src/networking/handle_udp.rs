use std::{
    collections::VecDeque,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use crate::{
    creatures::player::{AttackState, AttackType},
    networking::ServerConnection,
    prelude::*,
};

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(OnEnter(game_state), setup);

        app.add_systems(Update, read_udp_messages.run_if(in_state(game_state)));
    }
}

pub const UDP_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7188);

#[derive(ByteRepr, Debug, Clone)]
pub struct UdpToServer {
    pub id: u16,
    pub msg: UdpMsgToServer,
}

#[derive(ByteRepr, Debug, Clone)]
pub struct UdpToClient {
    /// The id of the last [UdpMsgToServer] sent by the client that was processed by the server at
    /// the time this [UdpMsgToClient] was constructed.
    pub last_processed_id: u16,
    pub msg: UdpMsgToClient,
}

#[derive(ByteRepr, Debug, Clone)]
pub enum UdpMsgToServer {
    Connect(ClientId),
    Disconnect,
    PlayerAttack { ty: AttackType },
    PlayerMovement { dir: [f32; 3] },
}

#[derive(ByteRepr, Debug, Clone)]
pub enum UdpMsgToClient {
    Connected,
    PlayerConnected { id: ClientId },
    PlayerDisconnected { id: ClientId },
    PlayerAttack { id: ClientId, ty: AttackType },
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Udp {
    next_id: u16,
    msg_cache: VecDeque<UdpToServer>,
    #[deref]
    com: UdpCommunicator<UdpToServer, UdpToClient>,
}

impl Udp {
    pub fn write(&mut self, msg: UdpMsgToServer) {
        let msg = UdpToServer {
            id: self.next_id,
            msg,
        };
        self.msg_cache.push_back(msg.clone());
        self.com.write_ordered(msg);
        self.next_id = self.next_id.wrapping_add(1);
    }
}

fn setup(mut commands: Commands) {
    let mut udp = Udp::default();
    udp.com.connect(UDP_ADDR).unwrap();
    commands.insert_resource(udp);
}

fn read_udp_messages(mut udp: ResMut<Udp>, con: Res<ServerConnection>, mut commands: Commands) {
    udp.recv();
    while let Some(UdpToClient {
        last_processed_id,
        msg,
    }) = udp.read_ordered()
    {
        debug!("Received msg via UDP: {msg:?}, last_processed_id: {last_processed_id}");
        match msg {
            UdpMsgToClient::Connected => {
                let Some(client_id) = con.client_id else {
                    error!("ClientId not initialized, not spawning player");
                    continue;
                };
                commands.spawn((MainCharacter, client_id));
            }
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
            UdpMsgToClient::PlayerDisconnected { id } => {
                if let Some(entity) = con.clients.get(&id) {
                    debug!("Despawning player #{id:?} ({entity})");
                    commands.entity(*entity).despawn();
                }
            }
        }
    }
    if udp.last_send().elapsed().as_millis() > 200 {
        udp.write_heartbeat();
    }
    udp.send().unwrap();
}

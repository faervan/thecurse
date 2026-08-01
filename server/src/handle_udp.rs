use thecurse_core::{creatures::player::AttackState, networking::UDP_ADDR};

use crate::{clients::ConnectedClients, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup);

    app.add_systems(FixedUpdate, read_udp);
}

#[derive(Resource, Deref, DerefMut)]
pub struct Udp(MultiUdpCommunicator<UdpMsgToClient, UdpMsgToServer>);

fn setup(mut commands: Commands) {
    let com = MultiUdpCommunicator::bind(UDP_ADDR)
        .with_fake_unreliablity()
        .with_debug_logs();
    commands.insert_resource(Udp(com));
}

fn read_udp(mut udp: ResMut<Udp>, mut clients: ResMut<ConnectedClients>, mut commands: Commands) {
    udp.tick(|addr, mut com, mut delayed| {
        while let Some(msg) = com.read() {
            debug!("Received msg {msg:?} from {addr:?} via UDP");
            match msg {
                UdpMsgToServer::Connect(id) => {
                    let Some(entity) = clients.client_entities.get(&id) else {
                        continue;
                    };
                    commands.entity(*entity).insert(ClientAddr(addr));
                    clients.client_addrs.insert(id, addr);
                    clients.addr_clients.insert(addr, id);
                    com.write(UdpMsgToClient::Connected);
                    delayed.broadcast_except(UdpMsgToClient::PlayerConnected { id }, addr);
                }
                UdpMsgToServer::Hello(_) => {}
                UdpMsgToServer::PlayerAttack { ty } => {
                    let id = clients.addr_clients.get(&addr).unwrap();
                    clients.client_entities.get(id).unwrap();
                    if let Some(id) = clients.addr_clients.get(&addr)
                        && let Some(entity) = clients.client_entities.get(id)
                    {
                        debug!("attack! {id:?} does {ty:?}");
                        commands.entity(*entity).insert(AttackState::Attacking {
                            timer: Timer::new(ty.duration(), TimerMode::Once),
                            ty,
                        });
                    }
                }
            }
        }
    });
}

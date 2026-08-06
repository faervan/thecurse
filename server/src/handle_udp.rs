use thecurse_core::networking::UDP_ADDR;

use crate::{
    clients::ConnectedClients,
    player::actions::{PlayerMovementQueue, apply_action},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Udp>();

    app.add_systems(ServerBroadcast, read_udp);
}

#[derive(Resource, Deref, DerefMut)]
pub struct Udp {
    #[deref]
    inner: MultiUdpCommunicator<UdpMsgToClient, UdpMsgToServer>,
    pub clients: ConnectedClients,
}

impl Default for Udp {
    fn default() -> Self {
        Self {
            inner: MultiUdpCommunicator::bind(UDP_ADDR)
                .with_fake_delay(20..25)
                .with_fake_drop(0.05)
                .with_fake_corruption(0.01),
            clients: ConnectedClients::default(),
        }
    }
}

impl Udp {
    pub fn flush_pending_messages(&mut self) {
        self.clients.flush_pending_messages(&mut self.inner);
    }

    pub fn borrow_mut(
        &mut self,
    ) -> (
        &mut MultiUdpCommunicator<UdpMsgToClient, UdpMsgToServer>,
        &mut ConnectedClients,
    ) {
        (&mut self.inner, &mut self.clients)
    }

    pub fn remove_stale_clients(&mut self, commands: &mut Commands) {
        self.inner.retain(|com| {
            if com.last_seen().elapsed() > Duration::from_secs(5) {
                info!("Removing client {:?} due to inactivity", com.addr);
                if let Some(entity) = self.clients.remove(com.addr) {
                    commands.entity(entity).despawn();
                } else {
                    warn!("Failed to remove client: client does not exist anymore");
                }
                return false;
            }
            true
        });
    }
}

fn read_udp(mut udp: ResMut<Udp>, mut commands: Commands) {
    let (com, clients) = udp.borrow_mut();
    com.recv(|mut com: UdpCommunicatorMut<_, _>| {
        while let Some(msg) = com.read_ordered() {
            match msg {
                UdpMsgToServer::Connect(id) => {
                    debug!("Received msg {msg:?} from {:?} via UDP", com.addr);
                    let entity = commands
                        .spawn((
                            Player,
                            Name::new(format!("Player #{}", id.0)),
                            id,
                            ClientAddr(com.addr),
                            Transform::from_translation(Vec3::Y),
                            PlayerMovementQueue::default(),
                        ))
                        .id();
                    clients.insert(id, u16::MAX, com.addr, entity, Vec3::Y.to_array());
                    com.write_ordered(UdpMsgToClient::Connected {
                        translation: Vec3::Y.to_array()
                    });
                }
                UdpMsgToServer::Disconnect => {
                    debug!("Received msg {msg:?} from {:?} via UDP", com.addr);
                    if let Some(entity) = clients.remove(com.addr) {
                        commands.entity(entity).despawn();
                    }
                }
                UdpMsgToServer::Ping { id } => com.write_ordered(UdpMsgToClient::Ping { id }),
                UdpMsgToServer::Action { id, action } => {
                    let id = clients.update_last_msg(id, &com.addr).unwrap();
                    match action {
                        PlayerAction::Attack { ty, .. } => {
                            debug!("attack! {id:?} does {ty:?}");
                        }
                        PlayerAction::Movement { origin, direction, destination, duration_secs } => {
                            debug!("Player {id:?} moves for {duration_secs}s from {origin:?} to {destination:?} (direction: {direction:?})");
                        }
                    }
                    if let Some(entity) = clients.get_client_entity(&id) {
                        apply_action(action, entity, &mut commands);
                    }
                }
            }
        }
    });
    udp.remove_stale_clients(&mut commands);
    udp.flush_pending_messages();
    udp.send();
}

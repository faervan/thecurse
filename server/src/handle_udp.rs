use thecurse_core::{creatures::player::AttackState, networking::UDP_ADDR};

use crate::{clients::ConnectedClients, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Udp>();

    app.add_systems(FixedUpdate, read_udp);
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
                .with_fake_unreliablity()
                .with_debug_logs(),
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
            debug!("Received msg {msg:?} from {:?} via UDP", com.addr);
            match msg {
                UdpMsgToServer::Connect(id) => {
                    let entity = commands
                        .spawn((
                            Player,
                            Name::new(format!("Player #{}", id.0)),
                            id,
                            ClientAddr(com.addr),
                        ))
                        .id();
                    clients.insert(id, u16::MAX, com.addr, entity);
                    com.write_ordered(UdpMsgToClient::Connected);
                }
                UdpMsgToServer::Disconnect => {
                    let entity = clients.remove(com.addr).unwrap();
                    commands.entity(entity).despawn();
                }
                UdpMsgToServer::Action { id, action } => {
                    let id = clients.update_last_msg(id, &com.addr).unwrap();
                    match action {
                        PlayerAction::Attack { ty } => {
                            if let Some(entity) = clients.get_client_entity(&id) {
                                debug!("attack! {id:?} does {ty:?}");
                                commands.entity(entity).insert(AttackState::Attacking {
                                    timer: Timer::new(ty.duration(), TimerMode::Once),
                                    ty,
                                });
                            }
                        }
                        PlayerAction::Movement => unimplemented!(),
                    }
                }
            }
        }
    });
    udp.remove_stale_clients(&mut commands);
    udp.flush_pending_messages();
    udp.send();
}

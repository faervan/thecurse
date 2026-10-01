use dreamgame_core::environment::EnvironmentObj;
use mini_udp::context::{error_handlers::WarnOnError, resend_strategies::FixedResend};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    let settings = app.world().resource::<ServerSettings>();
    app.insert_resource(Udp::new(settings));

    app.add_systems(AfterServerBroadcast, read_udp);
}

pub(crate) type UdpCfg = UdpConfig<
    MsgToClient,
    MsgToServer,
    PROTOCOL_VERSION,
    WarnOnError,
    FixedResend,
    (ClientId, Entity),
>;

#[derive(Resource, Debug)]
pub struct Udp {
    inner: MultiUdpCommunicator<UdpCfg>,
    pub server_broadcast_tick_id: u16,
}

impl Udp {
    fn new(settings: &ServerSettings) -> Self {
        let client_data_init = {
            let mut last_client_id = u64::MAX;
            move |_addr| {
                last_client_id = last_client_id.wrapping_add(1);
                (ClientId(last_client_id), Entity::PLACEHOLDER)
            }
        };
        Self {
            inner: {
                let mut com = MultiUdpCommunicator::<UdpCfg>::bind_with(
                    ("0.0.0.0", settings.port_udp),
                    WarnOnError,
                    client_data_init,
                );

                let resend_handler = com.get_resend_handler_mut();
                // This is basically an arbitrary low interval, it is extended to
                // `SERVER_TIMESTEP * 4` anyway, because that's the frequency at which the `send`
                // method is called.
                resend_handler.set_resend_interval(SERVER_TIMESTEP);

                // We manage disconnecting ourselves.
                com.get_timeout_mut().connection_timeout = Duration::ZERO;

                #[cfg(debug_assertions)]
                {
                    com = com
                        .with_fake_delay(20..25)
                        .with_fake_drop(0.05)
                        .with_fake_corruption(0.01);
                }
                com
            },
            server_broadcast_tick_id: 0,
        }
    }

    pub fn write_unreliable(&mut self, msg: MsgToClient, addr: &SocketAddr) {
        if let Some(mut com) = self.inner.get_mut(addr) {
            com.write(msg);
        } else {
            warn!("Tried to write {msg:?} to {addr:?}, but it is not connected");
        }
    }

    pub fn broadcast_unreliable(&mut self, msg: MsgToClient) {
        self.inner.for_each(|mut com| {
            com.write(msg.clone());
        });
    }

    pub fn broadcast_unreliable_except(&mut self, msg: MsgToClient, exception: ClientId) {
        self.inner.for_each(|mut com| {
            if com.data.0 != exception {
                com.write(msg.clone());
            }
        });
    }

    pub fn broadcast_ordered(&mut self, msg: MsgToClient) {
        self.inner.for_each(|mut com| {
            com.write_ordered(msg.clone());
        });
    }

    pub fn broadcast_ordered_except(&mut self, msg: MsgToClient, exception: ClientId) {
        self.inner.for_each(|mut com| {
            if com.data.0 != exception {
                com.write_ordered(msg.clone());
            }
        });
    }

    fn remove_stale_clients(&mut self, commands: &mut Commands) {
        self.inner.retain(|com| {
            if com.last_seen().elapsed() > Duration::from_secs(5) {
                debug!("Removing client {:?} due to inactivity", com.addr);
                commands.entity(com.data.1).try_despawn();
                return false;
            }
            true
        });
    }
}

fn read_udp(
    mut udp: ResMut<Udp>,
    mut commands: Commands,
    environment: Query<(&EnvironmentObj, &Transform), Without<Client>>,
    mut players: Query<(&mut Client, &mut Transform)>,
) {
    let server_tick_id = udp.server_broadcast_tick_id;
    udp.server_broadcast_tick_id = udp.server_broadcast_tick_id.wrapping_add(1);
    udp.inner.for_each(|mut com| {
        com.set_unreliable_packet_header_message(Some(MsgToClient::ServerTick { server_tick_id }))
            .unwrap()
    });

    udp.inner.recv(
        |mut com: UdpCommunicatorMut<UdpCfg>, mut con: ConnectionCommands| {
            if com.just_connected() {
                let id = com.data.0;
                let state = PlayerState {
                    translation: Vec3::Y.to_array(),
                };

                let entity = commands
                    .spawn_scene(Client::new(id, com.addr, state).into_scene())
                    .id();
                com.data.1 = entity;

                debug!("{:?} connected as {id:?}, {entity}", com.addr);
                com.write_ordered(MsgToClient::Connected {
                    id,
                    state: PlayerState {
                        translation: Vec3::Y.to_array(),
                    },
                });
                for (client, pos) in &players {
                    com.write_ordered(MsgToClient::PlayerInfo {
                        id: client.id,
                        state: PlayerState {
                            translation: pos.translation.to_array(),
                        },
                    });
                }
                for (obj, pos) in &environment {
                    com.write_ordered(MsgToClient::EnvironmentObj {
                        kind: *obj,
                        translation: pos.translation.to_array(),
                    });
                }
                return;
            }

            let entity = com.data.1;
            let Ok((mut client, _pos)) = players.get_mut(entity) else {
                return;
            };
            while let Some(msg) = com.read().or_else(|| com.read_ordered()) {
                match msg {
                    MsgToServer::Ping { id } => {
                        com.write(MsgToClient::PingResponse { id });
                    }
                    MsgToServer::Disconnect => {
                        debug!("Received msg {msg:?} from {:?} via UDP", com.addr);
                        commands.entity(entity).despawn();
                        con.disconnect();
                        break;
                    }
                    MsgToServer::Action { id, action } => {
                        client.read_action(id, action);
                    }
                }
            }
        },
    );

    for (mut client, mut transform) in &mut players {
        client.process_actions(&mut transform);
        client.broadcast(&mut udp);
    }

    udp.remove_stale_clients(&mut commands);
    udp.inner.send();
}

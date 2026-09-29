use crate::prelude::*;

pub fn plugin(app: &mut App) {
    let settings = app.world().resource::<ServerSettings>();
    app.insert_resource(Udp::new(settings));

    app.add_systems(AfterServerBroadcast, read_udp);
}

#[derive(Resource, Debug)]
pub struct Udp {
    inner: MultiUdpCommunicator<UdpServerCfg>,
    pub clients: ConnectedClients,
    pub server_broadcast_tick_id: u16,
}

impl Udp {
    fn new(settings: &ServerSettings) -> Self {
        Self {
            inner: {
                let mut com =
                    MultiUdpCommunicator::<UdpServerCfg>::bind(("0.0.0.0", settings.port_udp));

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
            clients: ConnectedClients::default(),
            server_broadcast_tick_id: 0,
        }
    }

    pub fn borrow_mut(
        &mut self,
    ) -> (
        &mut MultiUdpCommunicator<UdpServerCfg>,
        &mut ConnectedClients,
    ) {
        (&mut self.inner, &mut self.clients)
    }

    fn flush_pending_messages(&mut self) {
        self.clients
            .flush_pending_messages(&mut self.inner, self.server_broadcast_tick_id);
        self.server_broadcast_tick_id = self.server_broadcast_tick_id.wrapping_add(1);
    }

    fn remove_stale_clients(&mut self, commands: &mut Commands) {
        self.inner.retain(|com| {
            if com.last_seen().elapsed() > Duration::from_secs(5) {
                debug!("Removing client {:?} due to inactivity", com.addr);
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

fn read_udp(mut udp: ResMut<Udp>, mut commands: Commands, players: Query<(&ClientId, &Transform)>) {
    let (com, clients) = udp.borrow_mut();
    com.recv(
        |mut com: UdpCommunicatorMut<_>, just_connected: JustConnected| {
            if *just_connected {
                let id = ClientId(clients.next_client_id);
                clients.next_client_id = clients.next_client_id.wrapping_add(1);

                let entity = commands
                    .spawn((
                        Player,
                        Name::new(format!("Player #{}", id.0)),
                        id,
                        ClientAddr(com.addr),
                        Transform::from_translation(Vec3::Y),
                        // PlayerMovementQueue::default(),
                        // PlayerBroadcast {
                        //     first_movement_after_idle: true,
                        //     ..Default::default()
                        // },
                    ))
                    .id();

                debug!("{:?} connected as {id:?}, {entity}", com.addr);
                clients.insert(id, com.addr, entity, Vec3::Y.to_array());
                com.write_ordered(MsgToClient::Connected {
                    id,
                    translation: Vec3::Y.to_array(),
                });
                for (id, pos) in players {
                    com.write_ordered(MsgToClient::PlayerInfo {
                        id: *id,
                        translation: pos.translation.to_array(),
                    });
                }
                return;
            }
            while let Some(msg) = com.read_ordered().or_else(|| com.read()) {
                match msg {
                    MsgToServer::Ping { id } => {
                        com.write(MsgToClient::PingResponse { id });
                    }
                    MsgToServer::Disconnect => {
                        debug!("Received msg {msg:?} from {:?} via UDP", com.addr);
                        if let Some(entity) = clients.remove(com.addr) {
                            commands.entity(entity).despawn();
                        }
                    }
                }
            }
        },
    );
    udp.remove_stale_clients(&mut commands);
    udp.flush_pending_messages();
    udp.inner.send();
}

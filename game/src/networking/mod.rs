use crate::{environment::rasterized_grid_obj_scene, player::InnerMainCharacter, prelude::*};

mod action_queue;
mod con_info;

pub use con_info::ConnectionInfo;
use dreamgame_core::environment::EnvironmentObj;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::Game),
        |mut commands: Commands, settings: Res<GameSettings>| {
            commands.insert_resource(Udp::new(&settings))
        },
    );
    app.add_plugins(helpers::resource_in_state::<ConnectedClients>(
        AppState::Game,
    ));
    app.add_systems(OnEnter(Connected(false)), |mut commands: Commands| {
        commands.insert_resource(ConnectedClients::default());
    });

    app.add_systems(
        FixedUpdate,
        (
            (send_ping, ConnectionInfo::send_actions).run_if(in_state(Connected(true))),
            tick_udp,
        )
            .chain()
            .run_if(in_state(AppState::Game))
            .in_set(PhysicsSystems::Last),
    );
}

pub type UdpCfg = UdpContext<MsgToServer, MsgToClient, PROTOCOL_VERSION>;

#[derive(Resource)]
pub struct Udp {
    com: UdpCommunicator<UdpCfg>,
    server_tick_id: u16,
    pending_pings: RingBuffer<Instant, 8>,
    pub received_pings: RingBuffer<Duration, 16>,
}

#[derive(Resource, Reflect, Debug, Default, Deref, DerefMut)]
#[reflect(Resource)]
pub struct ConnectedClients {
    inner: HashMap<ClientId, Entity>,
}

impl Udp {
    fn new(settings: &GameSettings) -> Self {
        Self {
            com: {
                let mut com = UdpCommunicator::<UdpCfg>::default();
                com.connect((settings.addr.as_str(), settings.port_udp))
                    .unwrap();
                let resend_handler = com.get_resend_handler_mut();
                resend_handler.set_resend_interval(SERVER_TIMESTEP * 4);
                #[cfg(debug_assertions)]
                if !settings.no_fake_unreliability {
                    com = com
                        .with_fake_delay(35..45)
                        .with_fake_drop(0.05)
                        .with_fake_corruption(0.01);
                }
                com
            },
            server_tick_id: 0,
            pending_pings: RingBuffer::new(),
            received_pings: RingBuffer::new(),
        }
    }

    pub fn state(&self) -> &ConnectionState<<UdpCfg as MiniUdpContext>::ConnectionHandler> {
        self.com.state()
    }

    pub fn debug_state(&self) -> String {
        let state = format!(
            "State: {}",
            match self.com.state() {
                ConnectionState::Disconnected => "Disconnected",
                ConnectionState::Aborted { .. } => "Aborted ",
                ConnectionState::Connected { .. } => "Connected",
                ConnectionState::OutgoingConnectRequest { .. }
                | ConnectionState::IncomingConnectRequest { .. } => "Connecting",
            }
        );
        let last_seen = format!(
            "Last seen: {:4}ms ago",
            self.com.last_seen().elapsed().as_millis()
        );
        let last_send = format!(
            "Last send: {:4}ms ago",
            self.com.last_send().elapsed().as_millis()
        );
        let ping = self
            .received_pings
            .values()
            .map(|d| d.as_millis())
            .sum::<u128>()
            / self.received_pings.len().max(1) as u128;
        let loss = 100. / self.received_pings.max_len() as f32
            * (self.received_pings.max_len() - self.received_pings.len()) as f32;
        let ping = format!("Ping: {ping}ms ({loss:.0}% loss)");
        let server_tick = format!("Server tick: {}", self.server_tick_id);

        [state, last_seen, last_send, ping, server_tick].join("\n")
    }

    fn write_unreliable(&mut self, msg: MsgToServer) {
        self.com.write(msg);
    }

    pub fn disconnect(&mut self) {
        if self.com.state().is_connected() {
            info!("Sending disconnect notification to server");
            self.com.write_ordered(MsgToServer::Disconnect);
            self.com.send().unwrap();
        }
    }
}

fn tick_udp(
    mut udp: ResMut<Udp>,
    mut commands: Commands,
    mut connected_clients: ResMut<ConnectedClients>,
    mut con_info: Option<ResMut<ConnectionInfo>>,
    mut main_character: Option<
        Single<&mut Transform, (With<MainCharacter>, Without<InnerMainCharacter>)>,
    >,
    mut inner_characters: Option<Single<&mut Transform, With<InnerMainCharacter>>>,
    mut players: Query<&mut ScriptedPlayer>,
) {
    udp.com.recv();

    let mut outdated_server_tick = false;
    while let Some(msg) = udp.com.read().or_else(|| udp.com.read_ordered()) {
        match msg {
            MsgToClient::ServerTick { server_tick_id } => {
                if wrapping_gt(udp.server_tick_id, server_tick_id, u16::MAX / 2) {
                    outdated_server_tick = true;
                } else {
                    outdated_server_tick = false;
                    udp.server_tick_id = server_tick_id;
                }
            }
            MsgToClient::PingResponse { id } => {
                if let Some(send_instant) = udp.pending_pings.take(id) {
                    udp.received_pings.insert(id, send_instant.elapsed());
                } else {
                    debug!("Ping #{id} had a RTT longer than 800ms");
                }
            }
            MsgToClient::PlayerAction {
                id,
                action_id,
                action,
            } => {
                if outdated_server_tick {
                    continue;
                }
                todo!()
            }
            MsgToClient::StateUpdate {
                last_processed_action,
                state,
            } => {
                if let Some(con) = &mut con_info
                    && wrapping_gt(
                        last_processed_action,
                        con.last_processed_action,
                        u16::MAX / 2,
                    )
                    && let Some(transform) = &mut main_character
                    && let Some(inner_transform) = &mut inner_characters
                {
                    con.ack_action(last_processed_action, state, transform, inner_transform);
                }
            }
            MsgToClient::PlayerStateUpdate { id, state } => {
                if outdated_server_tick {
                    continue;
                }
                if let Some(entity) = connected_clients.get(&id)
                    && let Ok(mut player) = players.get_mut(*entity)
                {
                    player.push_state(udp.server_tick_id, state);
                }
            }
            MsgToClient::Connected {
                id,
                state,
                server_tick_id,
            } => {
                udp.server_tick_id = server_tick_id;

                let PlayerState { translation } = state;
                let translation = Vec3::from_array(translation);
                let entity = commands
                    .spawn_scene(bsn! {
                        @MainCharacter {
                            id
                        }
                        Transform::from_translation(translation)
                    })
                    .id();
                debug!("Connected as {id}, {entity}");
                commands.insert_resource(ConnectionInfo::new(id, entity, state));
                commands.set_state_if_neq(Connected(true));
            }
            MsgToClient::EnvironmentObj { kind, translation } => match kind {
                EnvironmentObj::RasterizedGrid => {
                    commands.run_system_cached_with(
                        rasterized_grid_obj_scene.pipe(spawn_obj_scene),
                        Vec3::from_array(translation),
                    );
                }
            },
            MsgToClient::PlayerInfo { id, state } => {
                debug!("Player {id} is also connected");
                spawn_player(
                    &mut connected_clients,
                    &mut commands,
                    id,
                    state,
                    // TODO! This server tick may be invalid, as this message can be read before
                    // MsgToClient::Connected.
                    udp.server_tick_id,
                );
            }
            MsgToClient::PlayerConnected { id, state } => {
                debug!("Player {id} send connect");
                spawn_player(
                    &mut connected_clients,
                    &mut commands,
                    id,
                    state,
                    // TODO! This server tick may be invalid, as this message can be read before
                    // MsgToClient::Connected.
                    udp.server_tick_id,
                );
            }
            MsgToClient::PlayerDisconnected { id } => {
                debug!("Player {id} send disconnect");
                if let Some(entity) = connected_clients.remove(&id) {
                    debug!("Despawning player {id}, {entity}");
                    commands.entity(entity).despawn();
                }
            }
        }
    }

    if let Err(e) = udp.com.send() {
        match e {
            MiniUdpError::NotConnected => {}
            _ => {
                error!("{e}");
            }
        }
    }
}

fn spawn_player(
    mapping: &mut ConnectedClients,
    commands: &mut Commands,
    id: ClientId,
    state: PlayerState,
    server_tick_id: u16,
) {
    let translation = Vec3::from_array(state.translation);
    let entity = commands
        .spawn((
            Name::new(format!("Player {id}")),
            Transform::from_translation(translation),
            ScriptedPlayer::new(id, state, server_tick_id),
        ))
        .id();
    debug!("Spawning player {id} as {entity}");
    mapping.insert(id, entity);
}

fn send_ping(mut udp: ResMut<Udp>, time: Res<Time>, mut timer: Local<Timer>) {
    if timer.duration().is_zero() {
        *timer = Timer::new(Duration::from_millis(100), TimerMode::Repeating);
    }
    timer.tick(time.delta());
    if timer.just_finished() {
        let id = udp.pending_pings.push(Instant::now());
        udp.com.write(MsgToServer::Ping { id });
    }
}

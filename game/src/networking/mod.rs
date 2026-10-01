use crate::{
    environment::rasterized_grid_obj_scene, networking::action_queue::PlayerAction, prelude::*,
};

mod action_queue;
mod con_info;

pub use con_info::ConnectionInfo;
use dreamgame_core::environment::EnvironmentObj;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ServerUpdates>();

    app.add_systems(
        OnEnter(AppState::Game),
        |mut commands: Commands, settings: Res<GameSettings>| {
            commands.insert_resource(Udp::new(&settings))
        },
    );

    app.add_systems(
        Update,
        (
            (send_ping, ConnectionInfo::send_actions).run_if(in_state(Connected(true))),
            tick_udp,
            handle_server_updates.run_if(in_state(Connected(true))),
        )
            .chain()
            .run_if(in_state(AppState::Game)),
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

#[derive(Resource, Debug, Default)]
struct ServerUpdates {
    actions: Vec<(ClientId, PlayerAction)>,
    main_character_state: Option<(u16, PlayerState)>,
    player_updates: Vec<(ClientId, PlayerState)>,
    disconnects: Vec<ClientId>,
}

fn tick_udp(
    mut udp: ResMut<Udp>,
    mut server_updates: ResMut<ServerUpdates>,
    mut commands: Commands,
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
                if server_updates
                    .main_character_state
                    .is_none_or(|(id, _)| wrapping_gt(last_processed_action, id, u16::MAX / 2))
                {
                    server_updates.main_character_state = Some((last_processed_action, state));
                }
            }
            MsgToClient::PlayerStateUpdate { id, state } => {
                if outdated_server_tick {
                    continue;
                }
                server_updates.player_updates.push((id, state));
                // TODO
            }
            MsgToClient::Connected { id, state } => {
                let PlayerState { translation } = state;
                let translation = Vec3::from_array(translation);
                let entity = commands
                    .spawn((
                        MainCharacter::default(),
                        Transform::from_translation(translation),
                    ))
                    .id();
                debug!("Connected as {id:?}, {entity}");
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
                debug!("Player {id:?} is also connected");
                commands.run_system_cached_with(spawn_player, (id, state));
            }
            MsgToClient::PlayerConnected { id, state } => {
                debug!("Player {id:?} send connect");
                commands.run_system_cached_with(spawn_player, (id, state));
            }
            MsgToClient::PlayerDisconnected { id } => {
                debug!("Player {id:?} send disconnect");
                server_updates.disconnects.push(id);
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
    In((id, state)): In<(ClientId, PlayerState)>,
    mut commands: Commands,
    mut con: ResMut<ConnectionInfo>,
) {
    let translation = Vec3::from_array(state.translation);
    let entity = commands
        .spawn((
            Name::new(format!("Player #{}", id.0)),
            Transform::from_translation(translation),
            ScriptedPlayer {
                id,
                authoritative_state: state,
            },
        ))
        .id();
    debug!("Spawning player {id:?} as {entity}");
    con.clients.insert(id, entity);
}

fn handle_server_updates(
    mut commands: Commands,
    mut con: ResMut<ConnectionInfo>,
    mut server_updates: ResMut<ServerUpdates>,
    mut players: Query<(&mut ScriptedPlayer, &mut Transform)>,
) {
    for id in server_updates.disconnects.drain(..) {
        if let Some(entity) = con.clients.remove(&id) {
            debug!("Despawning player {id:?}, {entity}");
            commands.entity(entity).despawn();
        }
    }

    for (id, action) in server_updates.actions.drain(..) {
        if let Some(_entity) = con.clients.get(&id)
        // && let Ok(_queue) = players.get_mut(*entity)
        {
            match action {}
        }
    }

    for (client_id, state) in server_updates.player_updates.drain(..) {
        if let Some(entity) = con.clients.get(&client_id)
            && let Ok((mut player, mut transform)) = players.get_mut(*entity)
        {
            transform.translation = Vec3::from_array(state.translation);
            player.authoritative_state = state;
        }
    }

    if let Some((last_processed_action, state)) = server_updates.main_character_state.take() {
        con.ack_action(last_processed_action, state);
    }
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

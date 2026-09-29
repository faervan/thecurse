use crate::{
    networking::action_queue::{PlayerAction, PlayerActionQueue},
    prelude::*,
};

mod action_queue;
mod con_info;

pub use con_info::ConnectionInfo;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<NewPlayerActions>();

    app.add_systems(
        OnEnter(AppState::Game),
        |mut commands: Commands, settings: Res<GameSettings>| {
            commands.insert_resource(Udp::new(&settings))
        },
    );

    app.add_systems(
        Update,
        (
            send_ping.run_if(in_state(Connected(true))),
            tick_udp,
            handle_new_actions.run_if(resource_exists::<ConnectionInfo>),
        )
            .chain()
            .run_if(in_state(AppState::Game)),
    );
}

#[derive(Resource)]
pub struct Udp {
    com: UdpCommunicator<UdpClientCfg>,
    pending_pings: RingBuffer<Instant, 8>,
    pub received_pings: RingBuffer<Duration, 8>,
}

impl Udp {
    fn new(settings: &GameSettings) -> Self {
        Self {
            com: {
                let mut com = UdpCommunicator::<UdpClientCfg>::default();
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
            pending_pings: RingBuffer::new(),
            received_pings: RingBuffer::new(),
        }
    }

    pub fn state(&self) -> &ConnectionState<<UdpClientCfg as MiniUdpContext>::ConnectionHandler> {
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
        let ping = format!("Ping: {ping}ms");

        [state, last_seen, last_send, ping].join("\n")
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
struct NewPlayerActions {
    actions: Vec<(ClientId, PlayerAction)>,
    disconnects: Vec<ClientId>,
}

fn tick_udp(
    mut udp: ResMut<Udp>,
    mut new_actions: ResMut<NewPlayerActions>,
    mut commands: Commands,
) {
    udp.com.recv();

    while let Some(msg) = udp.com.read() {
        match msg {
            MsgToClient::PingResponse { id } => {
                if let Some(send_instant) = udp.pending_pings.take(id) {
                    udp.received_pings.insert(id, send_instant.elapsed());
                } else {
                    debug!("Ping #{id} had a RTT longer than 800ms");
                }
            }
            _ => todo!(),
        }
    }
    while let Some(msg) = udp.com.read_ordered() {
        match msg {
            MsgToClient::Connected { id, translation } => {
                let translation = Vec3::from_array(translation);
                let entity = commands
                    .spawn((
                        // MainCharacter::new(translation),
                        id,
                        Transform::from_translation(translation),
                    ))
                    .id();
                debug!("Connected as {id:?}, {entity}");
                commands.insert_resource(ConnectionInfo::new(id, entity));
            }
            MsgToClient::PlayerInfo { id, translation } => {
                debug!("Player {id:?} is also connected");
                commands.run_system_cached_with(spawn_player, (id, translation));
            }
            MsgToClient::PlayerConnected { id, translation } => {
                debug!("Player {id:?} send connect");
                commands.run_system_cached_with(spawn_player, (id, translation));
            }
            MsgToClient::PlayerDisconnected { id } => {
                debug!("Player {id:?} send disconnect");
                new_actions.disconnects.push(id);
            }
            MsgToClient::PingResponse { .. } => error!("{msg:?} should be unreliable"),
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
    In((id, translation)): In<(ClientId, [f32; 3])>,
    mut commands: Commands,
    mut con: ResMut<ConnectionInfo>,
) {
    let translation = Vec3::from_array(translation);
    let entity = commands
        .spawn((
            //Player,
            id,
            Transform::from_translation(translation),
        ))
        .id();
    debug!("Spawning player {id:?} as {entity}");
    con.clients.insert(id, entity);
}

fn handle_new_actions(
    mut commands: Commands,
    mut con: ResMut<ConnectionInfo>,
    mut new_actions: ResMut<NewPlayerActions>,
    mut players: Query<&mut PlayerActionQueue>,
) {
    for id in new_actions.disconnects.drain(..) {
        if let Some(entity) = con.clients.remove(&id) {
            debug!("Despawning player {id:?}, {entity}");
            commands.entity(entity).despawn();
        }
    }

    for (id, action) in new_actions.actions.drain(..) {
        if let Some(entity) = con.clients.get(&id)
            && let Ok(_queue) = players.get_mut(*entity)
        {
            match action {}
        }
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

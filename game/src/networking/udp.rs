use crate::{player::apply_action, prelude::*};

pub(super) fn plugin(app: &mut App) {
    let settings = app.world().resource::<GameSettings>();
    // Prevent crash when opening <F2> debug HUD before entering the game.
    app.insert_resource(Udp::new(settings));

    app.add_systems(
        OnEnter(AppState::Game),
        |mut commands: Commands, settings: Res<GameSettings>| {
            commands.insert_resource(Udp::new(&settings))
        },
    );

    app.add_systems(
        Update,
        (read_udp_messages, send_ping).run_if(in_state(AppState::Game)),
    );
}

#[derive(Resource)]
pub struct Udp {
    com: UdpCommunicator<UdpMsgToServer, UdpMsgToClient, PROTOCOL_VERSION>,
    last_processed_action: u16,
    next_ping_id: u16,
    pub last_pings: RingBuffer<Duration, 4>,
    pending_pings: VecDeque<(u16, Instant)>,
}

impl Udp {
    fn new(settings: &GameSettings) -> Self {
        Self {
            com: {
                let mut com = UdpCommunicator::default()
                    .connect((settings.addr.as_str(), settings.port_udp))
                    .unwrap();
                com = com.with_reliable_ordered_resend_interval(SERVER_TIMESTEP * 4);
                com = com.with_reliable_unordered_resend_interval(SERVER_TIMESTEP * 4);
                #[cfg(debug_assertions)]
                if !settings.no_fake_unreliability {
                    com = com
                        .with_fake_delay(35..45)
                        .with_fake_drop(0.05)
                        .with_fake_corruption(0.01);
                }
                com
            },
            last_processed_action: u16::MAX,
            next_ping_id: 0,
            last_pings: RingBuffer::new(),
            pending_pings: VecDeque::new(),
        }
    }
}

impl Udp {
    #[inline(always)]
    pub fn write(&mut self, msg: UdpMsgToServer) {
        self.com.write_ordered(msg);
    }

    pub fn send(&mut self) {
        if self.com.last_send().elapsed().as_millis() > 200 {
            self.com.write_heartbeat();
        }
        self.com.send().unwrap();
    }

    pub fn disconnect(&mut self) {
        info!("Sending disconnect notification to server");
        self.write(UdpMsgToServer::Disconnect);
        self.com.send().unwrap();
    }
}

fn read_udp_messages(
    mut udp: ResMut<Udp>,
    con: Res<ServerConnection>,
    mut commands: Commands,
    mut character: Query<&mut MainCharacter>,
) {
    let mut ping_ids = vec![];
    udp.com.recv();
    while let Some(msg) = udp.com.read_ordered() {
        match msg {
            UdpMsgToClient::Connected { translation } => {
                debug!("Received msg via UDP: {msg:?}");
                let Some((client_id, entity)) = con.client_id else {
                    error!("ClientId not initialized, not spawning player");
                    return;
                };

                let translation = Vec3::from_array(translation);
                commands.entity(entity).insert((
                    MainCharacter::new(translation),
                    client_id,
                    Transform::from_translation(translation),
                ));
            }
            UdpMsgToClient::PlayerConnected { id, translation } => {
                debug!("Received msg via UDP: {msg:?}");
                let translation = Vec3::from_array(translation);
                commands.spawn((Player, id, Transform::from_translation(translation)));
            }
            UdpMsgToClient::PlayerDisconnected { id } => {
                debug!("Received msg via UDP: {msg:?}");
                if let Some(entity) = con.clients.get(&id) {
                    debug!("Despawning player #{id:?} ({entity})");
                    commands.entity(*entity).despawn();
                }
            }
            UdpMsgToClient::Ping { id } => ping_ids.push(id),

            UdpMsgToClient::PlayerAction {
                client_id,
                server_broadcast_tick_id,
                action,
                last_processed_action,
            } => {
                udp.last_processed_action = last_processed_action;
                if let Some((id, entity)) = con.client_id
                    && id == client_id
                {
                    commands
                        .entity(entity)
                        .entry::<MainCharacter>()
                        .and_modify(move |mut m| {
                            m.handle_action(action, last_processed_action);
                        });
                } else if let Some(entity) = con.clients.get(&client_id) {
                    debug!("Received action by {client_id:?}: {action:?}");
                    apply_action(action, server_broadcast_tick_id, *entity, &mut commands);
                }
            }
        }
    }
    for ping_id in ping_ids {
        if let Some((id, start)) = udp.pending_pings.pop_front()
            && id == ping_id
        {
            udp.last_pings.push(start.elapsed());
        } else {
            warn!("Received invalid ping id: {ping_id}");
        }
    }
    if let Ok(mut character) = character.single_mut() {
        for msg in character.new_messages.drain(..) {
            udp.com.write_ordered(msg);
        }
    }
    udp.send();
}

fn send_ping(mut udp: ResMut<Udp>, time: Res<Time>, mut timer: Local<Timer>) {
    if timer.duration().is_zero() {
        *timer = Timer::new(Duration::from_millis(100), TimerMode::Repeating);
    }
    timer.tick(time.delta());
    if timer.just_finished() {
        let id = udp.next_ping_id;
        udp.pending_pings.push_back((id, Instant::now()));
        udp.next_ping_id = udp.next_ping_id.wrapping_add(1);
        udp.write(UdpMsgToServer::Ping { id });
    }
}

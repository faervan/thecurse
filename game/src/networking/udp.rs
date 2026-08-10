use thecurse_core::{networking::UDP_ADDR, utils::wrapping::wrapping_le};

use crate::{player::apply_action, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Game), |mut commands: Commands| {
        commands.insert_resource(Udp::default())
    });

    app.add_systems(
        Update,
        (read_udp_messages, send_ping).run_if(in_state(AppState::Game)),
    );
}

#[derive(Resource)]
pub struct Udp {
    next_id: u16,
    action_cache: VecDeque<(u16, PlayerAction)>,
    com: UdpCommunicator<UdpMsgToServer, UdpMsgToClient>,
    next_ping_id: u16,
    pub last_pings: RingBuffer<Duration, 4>,
    pending_pings: VecDeque<(u16, Instant)>,
}

impl Default for Udp {
    fn default() -> Self {
        Self {
            next_id: 0,
            action_cache: VecDeque::new(),
            com: UdpCommunicator::default()
                .connect(UDP_ADDR)
                .unwrap()
                .with_reliable_ordered_resend_interval(Duration::from_millis(50))
                .with_fake_delay(35..45)
                .with_fake_drop(0.05)
                .with_fake_corruption(0.01),
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

    pub fn write_action(&mut self, action: PlayerAction) {
        self.action_cache.push_back((self.next_id, action.clone()));
        let msg = UdpMsgToServer::Action {
            id: self.next_id,
            action,
        };
        self.com.write_ordered(msg);
        self.next_id = self.next_id.wrapping_add(1);
    }

    pub fn send(&mut self) {
        if self.com.last_send().elapsed().as_millis() > 200 {
            self.com.write_heartbeat();
        }
        self.com.send().unwrap();
    }

    pub fn recv_with<F>(&mut self, mut f: F)
    where
        F: FnMut(UdpMsgToClient),
    {
        self.com.recv();
        while let Some(msg) = self.com.read_ordered() {
            if let UdpMsgToClient::PlayerAction {
                last_processed_action,
                ..
            } = msg
            {
                while self
                    .action_cache
                    .pop_front_if(|m| wrapping_le(m.0, last_processed_action))
                    .is_some()
                {}
            }
            f(msg)
        }
    }

    pub fn disconnect(&mut self) {
        info!("Sending disconnect notification to server");
        self.write(UdpMsgToServer::Disconnect);
        self.com.send().unwrap();
    }
}

fn read_udp_messages(mut udp: ResMut<Udp>, con: Res<ServerConnection>, mut commands: Commands) {
    let mut ping_ids = vec![];
    udp.recv_with(|msg| match msg {
        UdpMsgToClient::Connected { translation } => {
            debug!("Received msg via UDP: {msg:?}");
            let Some(client_id) = con.client_id else {
                error!("ClientId not initialized, not spawning player");
                return;
            };
            commands.spawn((
                MainCharacter,
                client_id,
                Transform::from_translation(Vec3::from_array(translation)),
            ));
        }
        UdpMsgToClient::PlayerConnected { id, translation } => {
            debug!("Received msg via UDP: {msg:?}");
            commands.spawn((
                Player,
                id,
                Transform::from_translation(Vec3::from_array(translation)),
            ));
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
            client_id, action, ..
        } => {
            debug!("Received action by {client_id:?}: {action:?}");
            if let Some(entity) = con.clients.get(&client_id) {
                apply_action(action, *entity, &mut commands);
            }
        }
    });
    for ping_id in ping_ids {
        if let Some((id, start)) = udp.pending_pings.pop_front()
            && id == ping_id
        {
            udp.last_pings.push(start.elapsed());
        } else {
            warn!("Received invalid ping id: {ping_id}");
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

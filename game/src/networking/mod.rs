use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::Game),
        |mut commands: Commands, settings: Res<GameSettings>| {
            commands.insert_resource(Udp::new(&settings))
        },
    );

    app.add_systems(Update, tick_udp.run_if(in_state(AppState::Game)));
}

#[derive(Resource)]
pub struct Udp {
    com: UdpCommunicator<UdpClientCfg>,
    last_processed_action: u16,
    next_ping_id: u16,
    pub last_pings: RingBuffer<Duration, 4>,
    pending_pings: VecDeque<(u16, Instant)>,
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
            last_processed_action: u16::MAX,
            next_ping_id: 0,
            last_pings: RingBuffer::new(),
            pending_pings: VecDeque::new(),
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
            "Last seen: {}s ago",
            self.com.last_seen().elapsed().as_secs()
        );
        let last_send = format!(
            "Last send: {}s ago",
            self.com.last_send().elapsed().as_secs()
        );
        [state, last_seen, last_send].join("\n")
    }

    pub fn disconnect(&mut self) {
        if self.com.state().is_connected() {
            info!("Sending disconnect notification to server");
            self.com.write_ordered(MsgToServer::Disconnect);
            self.com.send().unwrap();
        }
    }
}

fn tick_udp(mut udp: ResMut<Udp>) {
    udp.com.recv();
    udp.com.send().unwrap();
}

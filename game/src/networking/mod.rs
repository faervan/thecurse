use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::Game),
        |mut commands: Commands, settings: Res<GameSettings>| {
            commands.insert_resource(Udp::new(&settings))
        },
    );
}

#[derive(Resource)]
pub struct Udp {
    com: UdpCommunicator<MsgToServer, (), PROTOCOL_VERSION>,
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

    pub fn disconnect(&mut self) {
        info!("Sending disconnect notification to server");
        self.com.write_ordered(MsgToServer::Disconnect);
        self.com.send().unwrap();
    }
}

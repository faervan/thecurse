use bevy::time::Stopwatch;
use dreamgame_core::player::PLAYER_MOVEMENT_SEND_INTERVAL;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, pull_back_action_stopwatch);
}

fn pull_back_action_stopwatch(time: Res<Time>, clients: Query<&mut Client>) {
    for mut client in clients {
        let elapsed = client.action_stopwatch.elapsed();
        let new = elapsed - time.delta().min(elapsed);
        client.action_stopwatch.set_elapsed(new);
    }
}

#[derive(Component, Debug, Clone)]
#[component(on_add, on_despawn)]
pub struct Client {
    pub id: ClientId,
    pub addr: SocketAddr,
    //
    pub state: PlayerState,
    action_stopwatch: Stopwatch,
    last_processed_action: u16,
    unprocessed_actions: RingBuffer<PlayerAction, 64>,
    distance_traveled: RingBuffer<f32>,
    send_state_to_client: bool,
    broadcast_state: bool,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            id: ClientId(0),
            addr: SocketAddr::from(([0; 4], 0)),
            state: PlayerState {
                translation: [0.; 3],
            },
            action_stopwatch: Stopwatch::new(),
            last_processed_action: u16::MAX,
            unprocessed_actions: RingBuffer::new(),
            distance_traveled: {
                let mut ring = RingBuffer::new();
                for _ in 0..ring.max_len() {
                    ring.push(0.45);
                }
                ring
            },
            send_state_to_client: false,
            broadcast_state: false,
        }
    }
}

impl Client {
    pub fn new(id: ClientId, addr: SocketAddr, state: PlayerState) -> Self {
        Self {
            id,
            addr,
            state,
            ..Default::default()
        }
    }

    pub fn into_scene(self) -> impl Scene {
        bsn! {
            #Player
            Name::new(format!("Player #{}", self.id.0))
            Player
            Transform::from_translation(Vec3::from_array(self.state.translation))
            template_value(self)
        }
    }

    pub fn read_action(&mut self, action_id: u16, action: PlayerAction) {
        self.send_state_to_client = true;
        if wrapping_gt(
            self.last_processed_action.wrapping_add(1),
            action_id,
            u16::MAX / 2,
        ) {
            // Action has been processed already
            return;
        }
        if let Some(i) = self.unprocessed_actions.keys().next()
            && wrapping_gt(action_id, i.wrapping_add(63), u16::MAX / 2)
        {
            error!(
                "Received action_id {action_id} from client #{}, \
                but action {i} is not processed yet!",
                self.id.0
            );
            return;
        }
        self.unprocessed_actions.insert(action_id, action);
    }

    pub fn process_actions(&mut self, transform: &mut Transform) {
        while let Some(action) = self
            .unprocessed_actions
            .take(self.last_processed_action.wrapping_add(1))
        {
            self.last_processed_action = self.last_processed_action.wrapping_add(1);
            self.broadcast_state = true;
            self.send_state_to_client = true;
            match action {
                PlayerAction::Movement { offset } => {
                    self.action_stopwatch.tick(PLAYER_MOVEMENT_SEND_INTERVAL);
                    if self.action_stopwatch.elapsed() > Duration::from_millis(150) {
                        warn!(
                            "Client {} moved too often, action_stopwatch: {:.2}s",
                            self.id,
                            self.action_stopwatch.elapsed().as_secs_f32()
                        );
                        continue;
                    }
                    let received_offset = Vec3::from_array(offset);
                    let mut offset = received_offset;

                    debug_assert_eq!(
                        self.distance_traveled.len(),
                        self.distance_traveled.max_len()
                    );
                    let max_distance_sum = 0.55 * self.distance_traveled.max_len() as f32;

                    let prev_distance_sum = self.distance_traveled.values().sum::<f32>();
                    let oldest_distance = self.distance_traveled.values().next().unwrap();

                    let max_offset_len = max_distance_sum - (prev_distance_sum - oldest_distance);
                    let max_offset_len = max_offset_len.min(0.6);
                    offset = offset.clamp_length_max(max_offset_len);

                    self.distance_traveled.push(offset.length());
                    transform.translation += offset;
                    if offset.distance(received_offset) > 0.05 {
                        debug!(
                            "Client {} moved (id #{}) by {:.2} to {:.2?} - average: {:.2}, corrected by {:.2}",
                            self.id,
                            self.last_processed_action,
                            offset.length(),
                            transform.translation,
                            self.distance_traveled.values().sum::<f32>()
                                / self.distance_traveled.max_len() as f32,
                            offset.distance(received_offset)
                        );
                    }
                    self.state.apply(PlayerAction::Movement {
                        offset: offset.to_array(),
                    });
                }
            }
        }
    }

    pub fn broadcast(&mut self, udp: &mut Udp) {
        if std::mem::take(&mut self.send_state_to_client) {
            udp.write_unreliable(
                MsgToClient::StateUpdate {
                    last_processed_action: self.last_processed_action,
                    state: self.state,
                },
                &self.addr,
            );
        }

        // TODO! Add broadcast acks later
        if std::mem::take(&mut self.broadcast_state) || true {
            udp.broadcast_unreliable_except(
                MsgToClient::PlayerStateUpdate {
                    id: self.id,
                    state: self.state,
                },
                self.id,
            );
        }
    }

    fn on_add(mut world: DeferredWorld, hook: HookContext) {
        let client = world.get::<Self>(hook.entity).unwrap();
        let id = client.id;
        let state = client.state;
        info!("Added client {id:?}");

        let mut udp = world.resource_mut::<Udp>();
        udp.broadcast_ordered_except(MsgToClient::PlayerConnected { id, state }, id);
    }

    fn on_despawn(mut world: DeferredWorld, hook: HookContext) {
        let id = world.get::<Self>(hook.entity).unwrap().id;
        info!("Removed client {id:?}");

        let mut udp = world.resource_mut::<Udp>();
        udp.broadcast_ordered(MsgToClient::PlayerDisconnected { id });
    }
}

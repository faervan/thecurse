use bevy::time::Stopwatch;

use crate::prelude::*;

pub fn plugin(_app: &mut App) {}

#[derive(Component, Debug, Clone)]
#[component(on_add, on_despawn)]
pub struct Client {
    pub id: ClientId,
    pub addr: SocketAddr,
    //
    pub state: PlayerState,
    connected_since: Instant,
    action_stopwatch: Stopwatch,
    last_processed_action: u16,
    unprocessed_actions: RingBuffer<PlayerAction, 64>,
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
            connected_since: Instant::now(),
            action_stopwatch: Stopwatch::new(),
            last_processed_action: u16::MAX,
            unprocessed_actions: RingBuffer::new(),
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
            match action {
                PlayerAction::Movement { destination } => {
                    let destination = Vec3::from_array(destination);
                    debug!(
                        "Client #{} moved by {:.2} from {:?} to {destination:?}",
                        self.id.0,
                        destination.distance(transform.translation),
                        transform.translation
                    );
                    transform.translation = destination;
                    self.state.apply(action);
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

        if std::mem::take(&mut self.broadcast_state) {
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

use crate::prelude::*;

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ConnectionInfo {
    pub last_processed_action: u16,
    authoritative_state: PlayerState,
    /// Last authoritative state, but used as "predicted_state" in [`Self::ack_action`].
    predicted_state: PlayerState,
    /// The state after having applied all (unacked) actions.
    pub predicted_future_state: PlayerState,
    next_action_id: u16,
    action_send_timer: Timer,
    unacked_actions: VecDeque<(u16, PlayerAction)>,
    //
    client_id: ClientId,
    entity: Entity,
}

impl ConnectionInfo {
    pub fn new(id: ClientId, entity: Entity, state: PlayerState) -> Self {
        Self {
            last_processed_action: u16::MAX,
            authoritative_state: state,
            predicted_state: state,
            predicted_future_state: state,
            next_action_id: 0,
            action_send_timer: Timer::new(Duration::from_millis(50), TimerMode::Repeating),
            unacked_actions: VecDeque::new(),
            client_id: id,
            entity,
        }
    }

    pub fn add_action(&mut self, action: PlayerAction) {
        self.unacked_actions
            .push_back((self.next_action_id, action));
        self.next_action_id = self.next_action_id.wrapping_add(1);
        self.action_send_timer.almost_finish();
    }

    pub fn send_actions(mut this: ResMut<Self>, time: Res<Time>, mut udp: ResMut<Udp>) {
        this.action_send_timer.tick(time.delta());
        if this.action_send_timer.just_finished() {
            for (id, action) in this.unacked_actions.iter().copied() {
                udp.write_unreliable(MsgToServer::Action { id, action });
            }
        }
    }

    pub fn ack_action(
        &mut self,
        id: u16,
        state: PlayerState,
        transform: &mut Transform,
        inner_transform: &mut Transform,
    ) {
        if wrapping_gt(id, self.last_processed_action, u16::MAX / 2) {
            self.last_processed_action = id;
            self.authoritative_state = state;

            while let Some((_action_id, action)) = self
                .unacked_actions
                .pop_front_if(|(action_id, _)| !wrapping_gt(*action_id, id, u16::MAX / 2))
            {
                self.predicted_state.apply(action);
            }

            let authoritative = Vec3::from_array(self.authoritative_state.translation);
            let predicted = Vec3::from_array(self.predicted_state.translation);
            let offset = authoritative - predicted;
            if offset.length() > 0.05 {
                debug!(
                    "\nstate mismatch!\npredicted: {:#?}\nauthoritative: {:#?}\ndiff: {}\n",
                    self.predicted_state,
                    self.authoritative_state,
                    offset.length()
                );
                transform.translation += offset;
                inner_transform.translation -= offset;
                self.predicted_future_state = self.authoritative_state;
                for (_, action) in &self.unacked_actions {
                    self.predicted_future_state.apply(*action);
                }
            }

            self.predicted_state = self.authoritative_state;
        }
    }
}

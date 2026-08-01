use thecurse_core::creatures::player::{AttackState, attack_state_changes};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (broadcast_attack, attack_state_changes).chain());
}

fn broadcast_attack(
    mut udp: ResMut<Udp>,
    query: Query<(&AttackState, &ClientId, &ClientAddr), Changed<AttackState>>,
) {
    for (state, id, addr) in query {
        if let AttackState::Attacking { timer, ty } = state
            && timer.elapsed().is_zero()
        {
            udp.breadcast_except(UdpMsgToClient::PlayerAttack { id: *id, ty: *ty }, **addr);
        }
    }
}

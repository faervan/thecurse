use thecurse_core::creatures::player::{AttackState, attack_state_changes};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (broadcast_attack, attack_state_changes).chain(),
    );
}

fn broadcast_attack(
    mut udp: ResMut<Udp>,
    query: Query<(&AttackState, &Transform, &ClientId), Changed<AttackState>>,
) {
    for (state, pos, id) in query {
        if let AttackState::Attacking { timer, ty } = state
            && timer.elapsed().is_zero()
        {
            udp.clients.broadcast_action(
                *id,
                PlayerAction::Attack {
                    ty: *ty,
                    translation: pos.translation.to_array(),
                    rotation: pos.rotation.to_array(),
                },
            );
        }
    }
}

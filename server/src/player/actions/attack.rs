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
            let (com, clients) = udp.borrow_mut();
            let mut iter = com.iter_mut();
            while let Some(com) = iter.next() {
                if com.addr == **addr {
                    continue;
                }
                clients.write_to_com(UdpMsgToClient::PlayerAttack { id: *id, ty: *ty }, com);
            }
        }
    }
}

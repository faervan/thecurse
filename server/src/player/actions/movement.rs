use thecurse_core::creatures::player::MOVEMENT_SPEED;

use crate::prelude::*;

#[derive(Component, Default, Debug, Deref, DerefMut)]
pub struct PlayerMovementQueue(VecDeque<PlayerMovement>);

#[derive(Debug)]
pub struct PlayerMovement {
    pub direction: Vec2,
    pub timer: Timer,
    pub action_id: u16,
}

pub fn plugin(app: &mut App) {
    app.add_systems(FixedLast, movement_simulation);
    app.add_systems(ServerBroadcast, movement_broadcast);
}

fn movement_simulation(
    time: Res<Time>,
    mut udp: ResMut<Udp>,
    query: Query<(
        &mut PlayerMovementQueue,
        &mut PlayerBroadcast,
        &mut LinearVelocity,
        &Transform,
        &ClientAddr,
    )>,
) {
    for (mut queue, mut broadcast, mut vel, pos, addr) in query {
        while let Some(PlayerMovement {
            direction,
            timer,
            action_id,
        }) = queue.get_mut(0)
        {
            let start = timer.elapsed().is_zero();

            broadcast.movement_changed = true;
            timer.tick(time.delta());

            if timer.is_finished() && !start {
                // let server_broadcast_tick_id = udp.server_broadcast_tick_id;
                if let Some(client) = udp.clients.get_mut(addr) {
                    client.last_processed_action = *action_id;
                    // client
                    //     .pending_messages
                    //     .push_back(UdpMsgToClient::PlayerAction {
                    //         client_id: client.id,
                    //         last_processed_action: client.last_processed_action,
                    //         server_broadcast_tick_id,
                    //         action: PlayerActionBroadcast::Movement {
                    //             destination: pos.translation.to_array(),
                    //             just_started: broadcast.first_movement_after_idle,
                    //         },
                    //     });
                }
                queue.pop_front();
                vel.x = 0.;
                vel.z = 0.;
                continue;
            }

            vel.x = direction.x * MOVEMENT_SPEED;
            vel.z = direction.y * MOVEMENT_SPEED;
            break;
        }
    }
}

fn movement_broadcast(
    mut udp: ResMut<Udp>,
    query: Query<(&mut PlayerBroadcast, &Transform, &ClientId)>,
) {
    for (mut broadcast, pos, client_id) in query {
        if !broadcast.movement_changed {
            broadcast.first_movement_after_idle = true;
            continue;
        }

        udp.clients.broadcast_action(
            *client_id,
            PlayerActionBroadcast::Movement {
                destination: pos.translation.to_array(),
                just_started: broadcast.first_movement_after_idle,
            },
        );

        broadcast.movement_changed = false;
        broadcast.first_movement_after_idle = false;
    }
}

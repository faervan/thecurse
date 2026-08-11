use thecurse_core::creatures::player::MOVEMENT_SPEED;

use crate::prelude::*;

#[derive(Component, Default, Debug, Deref, DerefMut)]
pub struct PlayerMovementQueue(VecDeque<PlayerMovement>);

#[derive(Debug)]
pub struct PlayerMovement {
    pub origin: Vec3,
    pub direction: Vec2,
    pub destination: Vec3,
    pub timer: Timer,
}

pub fn plugin(app: &mut App) {
    app.add_systems(FixedLast, movement_simulation);
    app.add_systems(ServerBroadcast, movement_broadcast);
}

const MAX_MOVEMENT_INACCURACY: f32 = 0.05;

fn movement_simulation(
    time: Res<Time>,
    query: Query<(
        &mut PlayerMovementQueue,
        &mut PlayerBroadcast,
        &mut Transform,
        &mut LinearVelocity,
    )>,
) {
    for (mut queue, mut broadcast, mut pos, mut vel) in query {
        if let Some(PlayerMovement {
            origin,
            direction,
            destination,
            timer,
        }) = queue.get_mut(0)
        {
            broadcast.movement_changed = true;
            timer.tick(time.delta());
            if timer.is_finished() {
                if destination.distance(pos.translation.with_y(0.)) * timer.duration().as_secs_f32()
                    < MAX_MOVEMENT_INACCURACY
                {
                    // pos.translation.x = destination.x;
                    // pos.translation.z = destination.z;
                } else {
                    warn!(
                        "client send invalid movement: inaccuracy is {}m, per duration: {}, client moved by {:?}, but I say {:?}, duration: {}ms",
                        (destination.distance(pos.translation.with_y(0.)) * 1000.).round() / 1000.,
                        (destination.distance(pos.translation.with_y(0.))
                            / timer.duration().as_secs_f32()
                            * 100.)
                            .round()
                            / 100.,
                        *destination - *origin,
                        pos.translation.with_y(0.) - *origin,
                        timer.duration().as_millis()
                    );
                }
                // TODO! only do it when below MAX_MOVEMENT_INACCURACY
                pos.translation.x = destination.x;
                pos.translation.z = destination.z;
                queue.pop_front();
                vel.x = 0.;
                vel.z = 0.;
                continue;
            }
            vel.x = direction.x * MOVEMENT_SPEED;
            vel.z = direction.y * MOVEMENT_SPEED;
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

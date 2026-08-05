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
    pub last_broadcast: Instant,
}

pub fn plugin(app: &mut App) {
    app.add_systems(FixedLast, movement_simulation);
}

const MAX_MOVEMENT_INACCURACY: f32 = 0.1;

fn movement_simulation(
    mut udp: ResMut<Udp>,
    time: Res<Time>,
    query: Query<(
        &mut PlayerMovementQueue,
        &mut Transform,
        &mut LinearVelocity,
        &ClientId,
    )>,
) {
    for (mut queue, mut pos, mut vel, id) in query {
        if let Some(PlayerMovement {
            origin,
            direction,
            destination,
            timer,
            last_broadcast,
        }) = queue.get_mut(0)
        {
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
                udp.clients.broadcast_action(
                    *id,
                    PlayerActionBroadcast::Movement {
                        destination: pos.translation.to_array(),
                        duration_secs: last_broadcast.elapsed().as_secs_f32(),
                    },
                );
                *last_broadcast = Instant::now();
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

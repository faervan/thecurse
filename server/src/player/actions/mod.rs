use thecurse_core::creatures::player::{AttackState, MOVEMENT_SPEED};

use crate::{clients::ConnectedClient, player::actions::movement::PlayerMovement, prelude::*};

mod attack;

mod movement;
pub use movement::PlayerMovementQueue;

pub fn plugin(app: &mut App) {
    app.add_plugins((attack::plugin, movement::plugin));
}

pub fn apply_action(
    action: PlayerAction,
    action_id: u16,
    client: &mut ConnectedClient,
    commands: &mut Commands,
) {
    match action {
        PlayerAction::Attack {
            ty,
            translation,
            rotation,
        } => {
            debug!("attack! {:?} does {ty:?}", client.id);
            client.last_processed_action = action_id;
            commands
                .entity(client.entity)
                .insert(AttackState::Attacking {
                    timer: Timer::new(ty.duration(), TimerMode::Once),
                    ty,
                })
                .entry::<Transform>()
                .and_modify(move |mut pos| {
                    pos.translation = Vec3::from_array(translation);
                    pos.rotation = Quat::from_array(rotation)
                });
        }
        PlayerAction::Movement {
            direction,
            duration_millis,
        } => {
            if duration_millis == 0 {
                client.last_processed_action = action_id;
                return;
            }
            debug!(
                "Player {:?} moves for {duration_millis}ms in direction {direction:?}",
                client.id
            );
            let direction = Vec2::from_array(direction).normalize_or_zero();
            commands
                .entity(client.entity)
                .entry::<Transform>()
                .and_modify(move |mut pos| {
                    pos.look_to(-Vec3::new(direction.x, 0., direction.y), Vec3::Y);
                })
                .entity()
                .entry::<LinearVelocity>()
                .and_modify(move |mut vel| {
                    vel.x = direction.x * MOVEMENT_SPEED;
                    vel.z = direction.y * MOVEMENT_SPEED;
                })
                .entity()
                .entry::<PlayerMovementQueue>()
                .and_modify(move |mut queue| {
                    queue.push_back(PlayerMovement {
                        direction,
                        timer: Timer::new(
                            Duration::from_millis(duration_millis as u64),
                            TimerMode::Once,
                        ),
                        action_id,
                    });
                });
        }
    }
}

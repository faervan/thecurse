use thecurse_core::creatures::player::{AttackState, MOVEMENT_SPEED};

use crate::{player::actions::movement::PlayerMovement, prelude::*};

mod attack;

mod movement;
pub use movement::PlayerMovementQueue;

pub fn plugin(app: &mut App) {
    app.add_plugins((attack::plugin, movement::plugin));
}

pub fn apply_action(action: PlayerAction, entity: Entity, commands: &mut Commands) {
    match action {
        PlayerAction::Attack {
            ty,
            translation,
            rotation,
        } => {
            commands
                .entity(entity)
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
            origin,
            direction,
            destination,
            duration_secs,
        } => {
            let origin = Vec3::from_array([origin[0], 0., origin[1]]);
            let destination = Vec3::from_array([destination[0], 0., destination[1]]);
            let direction = Vec2::from_array(direction).normalize_or_zero();
            commands
                .entity(entity)
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
                        origin,
                        direction,
                        destination,
                        timer: Timer::new(Duration::from_secs_f32(duration_secs), TimerMode::Once),
                    });
                });
        }
    }
}

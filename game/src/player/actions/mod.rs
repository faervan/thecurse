use thecurse_core::creatures::player::AttackState;

use crate::prelude::*;

mod aerial;
mod attack;
mod movement;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((movement::plugin, aerial::plugin, attack::plugin));

    app.add_message::<InterruptAction>();

    app.add_systems(
        Update,
        drive_scripted_player_movement.run_if(in_state(AppState::Game)),
    );
}

#[derive(Component, Reflect, Default, Debug, Deref, DerefMut)]
#[reflect(Component)]
struct ScriptedPlayerMovementQueue(VecDeque<ScriptedPlayerMovement>);

#[derive(Reflect, Debug)]
struct ScriptedPlayerMovement {
    destination: Vec3,
    origin: Option<Vec3>,
    timer: Timer,
}

#[derive(Message, Debug)]
pub enum InterruptAction {
    PlayerJumped,
}

pub fn apply_action(action: PlayerActionBroadcast, entity: Entity, commands: &mut Commands) {
    match action {
        PlayerActionBroadcast::Attack {
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
        PlayerActionBroadcast::Movement {
            destination,
            duration_secs,
        } => {
            let destination = Vec3::from_array(destination);
            commands
                .entity(entity)
                .entry::<ScriptedPlayerMovementQueue>()
                // TODO! Insert it before
                .or_default()
                .and_modify(move |mut queue| {
                    queue.push_back(ScriptedPlayerMovement {
                        destination,
                        origin: None,
                        timer: Timer::new(Duration::from_secs_f32(duration_secs), TimerMode::Once),
                    })
                })
                .entity()
                .entry::<Transform>()
                .and_modify(move |mut pos| {
                    let dir = pos.translation - destination;
                    pos.look_to(dir.with_y(0.), Vec3::Y);
                });
        }
    }
}

fn drive_scripted_player_movement(
    time: Res<Time>,
    query: Query<(&mut ScriptedPlayerMovementQueue, &mut Transform)>,
) {
    for (mut queue, mut pos) in query {
        if queue.len() > 2 {
            debug!("{} movement events in queue", queue.len());
        }
        let mut overshoot = Duration::ZERO;
        while let Some(movement) = queue.get_mut(0) {
            movement.timer.tick(overshoot);
            let origin = match movement.origin {
                Some(p) => p,
                None => {
                    movement.origin = Some(pos.translation);
                    pos.translation
                }
            };
            if movement.timer.remaining() < time.delta() {
                overshoot = time.delta() - movement.timer.remaining();
            }
            movement.timer.tick(time.delta());
            pos.translation = origin + (movement.destination - origin) * movement.timer.fraction();
            if movement.timer.is_finished() {
                queue.pop_front();
                continue;
            }
            break;
        }
    }
}

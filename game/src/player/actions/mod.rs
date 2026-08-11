use thecurse_core::{creatures::player::AttackState, utils::wrapping::wrapping_le};

use crate::prelude::*;

mod aerial;
mod attack;
mod movement;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((movement::plugin, aerial::plugin, attack::plugin));

    app.add_message::<InterruptAction>();

    app.add_systems(
        Update,
        add_player_puppet_components.run_if(in_state(AppState::Game)),
    );

    app.add_systems(
        Update,
        drive_scripted_player_movement.run_if(in_state(AppState::Game)),
    );
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct ScriptedPlayerMovementQueue {
    last_pos: Vec3,
    next_pos: Option<Vec3>,
    /// Measures the time of the current movement replication
    movement_timer: Timer,
    last_server_tick_id: u16,
    pending: VecDeque<ScriptedPlayerMovement>,
    smoothness_delay: Timer,
}

impl Default for ScriptedPlayerMovementQueue {
    fn default() -> Self {
        Self {
            last_pos: Vec3::ZERO,
            next_pos: None,
            movement_timer: Timer::new(Duration::ZERO, TimerMode::Once),
            last_server_tick_id: u16::MAX,
            pending: VecDeque::new(),
            smoothness_delay: {
                let mut timer = Timer::new(Duration::from_millis(50), TimerMode::Once);
                timer.finish();
                timer
            },
        }
    }
}

#[derive(Reflect, Debug)]
struct ScriptedPlayerMovement {
    server_ticks: u16,
    destination: Vec3,
}

#[derive(Message, Debug)]
pub enum InterruptAction {
    PlayerJumped,
}

fn add_player_puppet_components(
    mut commands: Commands,
    query: Query<(Entity, &Transform), (Added<Player>, Without<MainCharacter>)>,
) {
    for (entity, pos) in query {
        commands.entity(entity).insert(ScriptedPlayerMovementQueue {
            last_pos: pos.translation,
            ..Default::default()
        });
    }
}

pub fn apply_action(
    action: PlayerActionBroadcast,
    server_broadcast_tick_id: u16,
    entity: Entity,
    commands: &mut Commands,
) {
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
            just_started,
        } => {
            commands
                .entity(entity)
                .entry::<ScriptedPlayerMovementQueue>()
                .and_modify(move |mut queue| {
                    if wrapping_le(server_broadcast_tick_id, queue.last_server_tick_id) {
                        warn!(
                            "Movement event with id {server_broadcast_tick_id} took too \
                            long (last processed is {})",
                            queue.last_server_tick_id
                        );
                        return;
                    }
                    let server_ticks = match just_started {
                        true => 1,
                        false => server_broadcast_tick_id.wrapping_sub(queue.last_server_tick_id),
                    };
                    if server_ticks > 1 {
                        warn!("Processing {server_ticks} server ticks for movement at once");
                    }
                    queue.last_server_tick_id = server_broadcast_tick_id;

                    let destination = Vec3::from_array(destination);
                    if queue.pending.is_empty() && queue.next_pos.is_none() {
                        queue.smoothness_delay.reset();
                    }
                    queue.pending.push_back(ScriptedPlayerMovement {
                        server_ticks,
                        destination,
                    });
                });
        }
    }
}

fn drive_scripted_player_movement(
    time: Res<Time>,
    query: Query<(&mut ScriptedPlayerMovementQueue, &mut Transform)>,
) {
    for (mut queue, mut pos) in query {
        if !queue.smoothness_delay.is_finished() {
            queue.smoothness_delay.tick(time.delta());
            continue;
        }

        if let Some(next_pos) = queue.next_pos {
            queue.movement_timer.tick(time.delta());
            let fraction = queue.movement_timer.fraction();

            pos.translation = queue.last_pos.lerp(next_pos, fraction);

            if queue.movement_timer.just_finished() {
                queue.last_pos = next_pos;
                queue.next_pos = None;
            } else {
                continue;
            }
        }

        if let Some(ScriptedPlayerMovement {
            server_ticks,
            destination,
        }) = queue.pending.pop_front()
        {
            queue
                .movement_timer
                .set_duration(SERVER_TIMESTEP * 4 / server_ticks as u32);
            queue.movement_timer.reset();
            queue.next_pos = Some(destination);
        }
    }
}

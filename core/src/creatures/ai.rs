use bitflags::bitflags;

use crate::{creatures::goblin::Goblin, prelude::*};

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(
            Update,
            (update_creature_ai, move_towards_target)
                .chain()
                .run_if(in_state(game_state)),
        );
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct CreatureThing {
    pub detection_range: f32,
    pub max_aggro_range: f32,
    /// How far the create wants to be from the target
    pub target_distance: f32,
    pub speed: f32,
    pub hostile_towards: CreatureFaction,
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub enum CreatureAiState {
    #[default]
    Idle,
    Aggro {
        target: Entity,
        /// position of the targeted entity, synced by [`update_creature_ai`]
        position: Vec3,
    },
}

#[derive(Reflect, Debug)]
pub struct CreatureFaction(u32);

bitflags! {
    impl CreatureFaction: u32 {
        const PLAYER = 0b00000001;
        const GOBLIN = 0b00000010;
    }
}

fn update_creature_ai(
    query: Query<(&mut CreatureAiState, &CreatureThing, &Transform)>,
    players: Query<(Entity, &Transform), With<Player>>,
    goblins: Query<(Entity, &Transform), With<Goblin>>,
    creatures: Query<&Transform, With<Creature>>,
) {
    for (mut state, thing, creature_transform) in query {
        match &mut *state {
            CreatureAiState::Idle => {
                if thing.hostile_towards.is_empty() {
                    continue;
                }
                let mut closest = None;
                if thing.hostile_towards.contains(CreatureFaction::PLAYER) {
                    for (entity, transform) in players {
                        let distance = transform
                            .translation
                            .distance(creature_transform.translation);
                        if distance <= thing.detection_range
                            && closest.is_none_or(|(_, dist, _)| dist > distance)
                        {
                            closest = Some((entity, distance, transform.translation));
                        }
                    }
                }
                if thing.hostile_towards.contains(CreatureFaction::GOBLIN) {
                    for (entity, transform) in goblins {
                        let distance = transform
                            .translation
                            .distance(creature_transform.translation);
                        if distance <= thing.detection_range
                            && closest.is_none_or(|(_, dist, _)| dist > distance)
                        {
                            closest = Some((entity, distance, transform.translation));
                        }
                    }
                }
                if let Some((target, _, position)) = closest {
                    debug!("Creature got aggro towards {target}");
                    *state = CreatureAiState::Aggro { target, position };
                }
            }
            CreatureAiState::Aggro { target, position } => match creatures.get(*target) {
                Ok(transform) => {
                    if transform
                        .translation
                        .distance(creature_transform.translation)
                        > thing.max_aggro_range
                    {
                        debug!("Creature lost aggro, target out of range");
                        *state = CreatureAiState::Idle;
                    } else if transform.translation != *position {
                        *position = transform.translation;
                    }
                }
                Err(_) => {
                    debug!("Creature lost aggro, target {target} despawned.");
                    *state = CreatureAiState::Idle;
                }
            },
        }
    }
}

fn move_towards_target(
    query: Query<(
        &CreatureAiState,
        &CreatureThing,
        &mut Transform,
        &mut LinearVelocity,
    )>,
) {
    for (state, thing, mut transform, mut velocity) in query {
        let CreatureAiState::Aggro { position, .. } = *state else {
            return;
        };
        let mut direction = (position - transform.translation).with_y(0.);
        transform.look_to(-direction, Vec3::Y);
        if direction.length() <= thing.target_distance {
            direction = Vec3::ZERO;
        }
        velocity.0 = direction.normalize_or_zero() * thing.speed;
    }
}

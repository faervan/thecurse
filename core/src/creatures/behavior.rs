use crate::prelude::*;

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(
            Update,
            (find_targets, remove_targets).run_if(in_state(game_state)),
        );

        app.add_systems(
            Update,
            creature_move_towards_target.run_if(in_state(game_state)),
        );
    }
}

#[derive(Component, Reflect, Debug, Deref)]
#[reflect(Component)]
pub struct CreatureTarget(Entity);

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[component(on_add)]
pub struct CreatureLookForTarget {
    pub search_radius: f32,
    /// TODO!
    pub search_requires_los: bool,
    /// If the target is further away than this, the creature will loose it. Thus this value has to
    /// be smaller than `search_radius`.
    pub max_follow_distance: f32,
    pub entity_filter: GameLayer,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct CreatureSearchSensor {
    sensor: Entity,
    detected_entities: HashSet<Entity>,
}

impl CreatureLookForTarget {
    fn on_add(mut world: DeferredWorld, hook: HookContext) {
        let this = world.get::<Self>(hook.entity).unwrap();
        let radius = this.search_radius;
        let filter = this.entity_filter;

        let id = world
            .commands()
            .spawn((
                Name::new("Creature target search sensor"),
                Collider::sphere(radius),
                CollisionEventsEnabled,
                Sensor,
                CollisionLayers::new(GameLayer::DEFAULT, filter),
                ChildOf(hook.entity),
            ))
            .observe(
                |event: On<CollisionStart>,
                 sensors: Query<&ChildOf, With<Sensor>>,
                 mut creatures: Query<&mut CreatureSearchSensor>| {
                    debug!("Collision with {}", event.collider2);
                    let Ok(parent) = sensors.get(event.event_target()) else {
                        return;
                    };
                    let Ok(mut search) = creatures.get_mut(parent.0) else {
                        return;
                    };
                    search.detected_entities.insert(event.collider2);
                },
            )
            .id();
        world
            .commands()
            .entity(hook.entity)
            .insert(CreatureSearchSensor {
                sensor: id,
                detected_entities: HashSet::new(),
            });
    }
}

fn find_targets(
    mut commands: Commands,
    query: Query<
        (Entity, &mut CreatureSearchSensor, &Transform),
        (Changed<CreatureSearchSensor>, Without<CreatureTarget>),
    >,
    targets: Query<&Transform>,
) {
    for (entity, mut search, transform) in query {
        if search.detected_entities.is_empty() {
            continue;
        }
        if let Some((closest, _)) = search
            .detected_entities
            .drain()
            .filter_map(|id| {
                targets
                    .get(id)
                    .map(|pos| (id, pos.translation.distance(transform.translation)))
                    .ok()
            })
            .reduce(|(id, distance), (next_id, next_distance)| {
                if next_distance < distance {
                    (next_id, next_distance)
                } else {
                    (id, distance)
                }
            })
        {
            commands.entity(entity).insert(CreatureTarget(closest));
            debug!("Creature found target {closest}");
            commands
                .entity(search.sensor)
                .remove::<Collider>()
                .remove::<Sensor>();
        }
    }
}

fn remove_targets(
    mut commands: Commands,
    query: Query<(
        Entity,
        &CreatureTarget,
        &CreatureLookForTarget,
        &CreatureSearchSensor,
        &Transform,
    )>,
    targets: Query<&Transform>,
) {
    for (entity, target, config, search, transform) in query {
        if targets.get(**target).ok().is_none_or(|target_transform| {
            transform.translation.distance(target_transform.translation)
                > config.max_follow_distance
        }) {
            commands.entity(entity).remove::<CreatureTarget>();
            commands
                .entity(search.sensor)
                .insert((Collider::sphere(config.search_radius), Sensor));
            debug!("Creature {entity} lost its target!");
        }
    }
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
/// Make the creature walk towards its [`CreatureTarget`]. Once it reached the target, this
/// component will be removed and a [`CreatureReachedTarget`] event will be fired.
/// This will also rotate the creature towards the target.
pub struct CreatureMoveTowardsTarget {
    /// Gap between the creature and its target that is still interpreted as "creature reached
    /// target". Once the distance between the creature and the target is smaller or equal to this
    /// value, the target is considered to be reached by the creature.
    pub target_gap: f32,
    pub speed: f32,
}

#[derive(EntityEvent, Debug)]
pub struct CreatureReachedTarget(Entity);

fn creature_move_towards_target(
    mut commands: Commands,
    query: Query<(
        Entity,
        &Transform,
        &CreatureTarget,
        &CreatureMoveTowardsTarget,
        &mut LinearVelocity,
    )>,
    targets: Query<&Transform>,
) {
    for (entity, transform, target, move_to_target, mut velocity) in query {
        let Ok(target_transform) = targets.get(**target) else {
            continue;
        };
        let direction = (target_transform.translation - transform.translation).with_y(0.);
        let rotation = Quat::from_rotation_arc(Vec3::NEG_Z, -direction.normalize_or(Vec3::NEG_Z));
        commands.entity(entity).transition(rotation, 100);
        if direction.length() <= move_to_target.target_gap {
            if velocity.0 != Vec3::ZERO {
                velocity.0 = Vec3::ZERO;
            }
            commands
                .entity(entity)
                .remove::<CreatureMoveTowardsTarget>();
            commands.entity(entity).trigger(CreatureReachedTarget);
            continue;
        }
        velocity.0 = direction.normalize_or_zero() * move_to_target.speed;
    }
}

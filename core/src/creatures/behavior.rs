use crate::prelude::*;

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(
            Update,
            (find_targets, remove_targets).run_if(in_state(game_state)),
        );

        app.add_systems(
            Update,
            (
                insert_creature_navmesh_paths,
                update_creature_paths,
                creature_move_towards_target,
            )
                .run_if(in_state(game_state)),
        );
    }
}

#[derive(Component, Reflect, Debug, Deref)]
#[reflect(Component)]
pub struct CreatureTarget(Entity);

#[derive(EntityEvent)]
pub struct CreatureTargetFound(Entity);

#[derive(EntityEvent)]
pub struct CreatureTargetLost(Entity);

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[component(on_add)]
/// Will trigger a [`CreatureTargetFound`] event when a target was found and a
/// [`CreatureTargetLost`] when the target was lost.
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
            commands
                .entity(entity)
                .insert(CreatureTarget(closest))
                .trigger(CreatureTargetFound);
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
            commands
                .entity(entity)
                .remove::<CreatureTarget>()
                .trigger(CreatureTargetLost);
            if let Ok(mut entity_cmds) = commands.get_entity(search.sensor) {
                entity_cmds.insert((Collider::sphere(config.search_radius), Sensor));
            }

            debug!("Creature {entity} lost its target!");
        }
    }
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
/// Make the creature walk towards its [`CreatureTarget`]. Once it reached the target, this
/// component will be removed and a [`CreatureReachedTarget`] event will be fired.
/// If the target can't be reached, a [`CreatureTargetUnreachable`] event will be fired instead.
/// This will also rotate the creature towards the target.
pub struct CreatureMoveTowardsTarget {
    /// Gap between the creature and its target that is still interpreted as "creature reached
    /// target". Once the distance between the creature and the target is smaller or equal to this
    /// value, the target is considered to be reached by the creature.
    pub target_gap: f32,
    pub speed: f32,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(crate) struct CreatureNavmeshPath {
    pub(crate) path: Vec<Vec2>,
    pub(crate) start: Vec2,
    update_timer: Timer,
}

#[derive(EntityEvent, Debug)]
pub struct CreatureReachedTarget(Entity);

#[derive(EntityEvent, Debug)]
pub struct CreatureTargetUnreachable(Entity);

fn insert_creature_navmesh_paths(
    navmeshes: Res<Assets<NavMesh>>,
    mut commands: Commands,
    query: Query<
        (Entity, &Transform, &CreatureTarget),
        (
            With<CreatureMoveTowardsTarget>,
            Without<CreatureNavmeshPath>,
        ),
    >,
    targets: Query<&Transform>,
) {
    let Some(navmesh) = navmeshes.get(ManagedNavMesh::from_id(0)) else {
        warn!("No NavMesh available");
        return;
    };
    for (entity, transform, target) in query {
        let start = vec2(transform.translation.x, transform.translation.z);
        let Some(path) = get_creature_path(navmesh, start, &targets, **target) else {
            trigger_target_unreachable(&mut commands, entity);
            continue;
        };
        commands.entity(entity).try_insert(CreatureNavmeshPath {
            path,
            start,
            update_timer: Timer::new(Duration::from_millis(500), TimerMode::Repeating),
        });
    }
}

fn update_creature_paths(
    time: Res<Time>,
    navmeshes: Res<Assets<NavMesh>>,
    mut commands: Commands,
    query: Query<(
        Entity,
        &Transform,
        &CreatureTarget,
        &mut CreatureNavmeshPath,
    )>,
    targets: Query<&Transform>,
) {
    let delta = time.delta();
    let Some(navmesh) = navmeshes.get(ManagedNavMesh::from_id(0)) else {
        warn!("No NavMesh available");
        return;
    };
    for (entity, transform, target, mut navmesh_path) in query {
        navmesh_path.update_timer.tick(delta);
        if navmesh_path.update_timer.just_finished() {
            let start = vec2(transform.translation.x, transform.translation.z);
            let Some(path) = get_creature_path(navmesh, start, &targets, **target) else {
                trigger_target_unreachable(&mut commands, entity);
                continue;
            };
            navmesh_path.path = path;
        }
    }
}

fn get_creature_path(
    navmesh: &NavMesh,
    start: Vec2,
    targets: &Query<&Transform>,
    target: Entity,
) -> Option<Vec<Vec2>> {
    let target_transform = targets.get(target).ok()?;
    let mut end = vec2(
        target_transform.translation.x,
        target_transform.translation.z,
    );
    if !navmesh.is_in_mesh(end) {
        let closest = navmesh.get().get_closest_point(end)?;
        end = closest.position();
    }
    let path = navmesh.path(start, end)?;
    Some(path.path.into_iter().rev().collect())
}

fn trigger_target_unreachable(commands: &mut Commands, target: Entity) {
    commands
        .entity(target)
        .remove::<CreatureMoveTowardsTarget>()
        .trigger(CreatureTargetUnreachable);
}

fn creature_move_towards_target(
    mut commands: Commands,
    query: Query<(
        Entity,
        &Transform,
        &mut CreatureNavmeshPath,
        &CreatureMoveTowardsTarget,
        &mut LinearVelocity,
    )>,
) {
    for (entity, transform, mut path, move_to_target, mut velocity) in query {
        let next_step = path.path.last().unwrap();

        let next = vec3(next_step.x, 0., next_step.y);
        let direction = (next - transform.translation).with_y(0.);
        let rotation = Quat::from_rotation_arc(Vec3::NEG_Z, -direction.normalize_or(Vec3::NEG_Z));
        commands.entity(entity).transition(rotation, 100);

        if path.path.len() == 1 && direction.length() <= move_to_target.target_gap {
            trigger_target_reached(&mut commands, entity, velocity);
            continue;
        } else if direction.length() > (next_step - path.start).length() || direction.length() < 0.5
        {
            path.path.pop();
            if path.path.is_empty() {
                trigger_target_reached(&mut commands, entity, velocity);
                continue;
            }
        }
        velocity.0 = direction.normalize_or_zero() * move_to_target.speed;
    }
}

fn trigger_target_reached(
    commands: &mut Commands,
    target: Entity,
    mut velocity: Mut<LinearVelocity>,
) {
    if velocity.0 != Vec3::ZERO {
        velocity.0 = Vec3::ZERO;
    }
    if let Ok(mut cmds) = commands.get_entity(target) {
        cmds.remove::<(CreatureMoveTowardsTarget, CreatureNavmeshPath)>()
            .trigger(CreatureReachedTarget);
    }
}

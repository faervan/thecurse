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
                update_direction_to_target,
                creature_move_away_from_target,
            )
                .run_if(in_state(game_state)),
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
            commands.entity(entity).try_insert(CreatureTarget(closest));
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
                .try_insert((Collider::sphere(config.search_radius), Sensor));

            debug!("Creature {entity} lost its target!");
        }
    }
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
/// Make the creature walk towards its [`CreatureTarget`].
/// If the target can't be reached, a [`CreatureTargetUnreachable`] event will be fired instead.
/// This will also rotate the creature towards the target.
pub struct CreatureMoveTowardsTarget {
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
        .remove::<(CreatureMoveTowardsTarget, CreatureNavmeshPath)>()
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
        let Some(next_step) = path.path.last() else {
            continue;
        };

        let next = vec3(next_step.x, 0., next_step.y);
        let direction = (next - transform.translation).with_y(0.);
        let rotation = Quat::from_rotation_arc(Vec3::NEG_Z, -direction.normalize_or(Vec3::NEG_Z));
        commands.entity(entity).transition(rotation, 100);

        if direction.length() < 0.5 || direction.length() > (next_step - path.start).length() {
            path.path.pop();
        }
        velocity.0 = direction.normalize_or_zero() * move_to_target.speed;
    }
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
/// Make the creature walk away from its [`CreatureTarget`].
/// This will also rotate the creature towards the target.
pub struct CreatureMoveAwayFromTarget {
    pub speed: f32,
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub(super) struct CreatureDirectionToTarget {
    direction: Vec3,
    timer: Timer,
}

fn update_direction_to_target(
    time: Res<Time>,
    mut commands: Commands,
    query: Query<
        (
            Entity,
            Option<&mut CreatureDirectionToTarget>,
            &CreatureTarget,
            &Transform,
        ),
        With<CreatureMoveAwayFromTarget>,
    >,
    targets: Query<&Transform>,
    mut fallback_rotation_offset: Local<f32>,
) {
    for (entity, direction_maybe, target, transform) in query {
        let Some(mut direction) = direction_maybe else {
            let Ok(target_transform) = targets.get(**target) else {
                continue;
            };
            commands
                .entity(entity)
                .try_insert(CreatureDirectionToTarget {
                    direction: get_direction(
                        transform,
                        target_transform,
                        &mut fallback_rotation_offset,
                    ),
                    timer: Timer::new(Duration::from_millis(200), TimerMode::Repeating),
                });
            continue;
        };
        direction.timer.tick(time.delta());
        if direction.timer.just_finished()
            && let Ok(target_transform) = targets.get(**target)
        {
            direction.direction =
                get_direction(transform, target_transform, &mut fallback_rotation_offset);
        }
    }
}

fn get_direction(
    from: &Transform,
    to: &Transform,
    fallback_rotation_offset: &mut Local<f32>,
) -> Vec3 {
    let fallback = Quat::from_rotation_y(fallback_rotation_offset.sin());
    **fallback_rotation_offset += 1.;
    (to.translation - from.translation)
        .with_y(0.)
        .normalize_or(fallback * Vec3::NEG_Z)
}

fn creature_move_away_from_target(
    mut commands: Commands,
    query: Query<(
        Entity,
        &CreatureMoveAwayFromTarget,
        &CreatureDirectionToTarget,
        &mut LinearVelocity,
    )>,
) {
    for (entity, move_from_target, direction, mut velocity) in query {
        let rotation = Quat::from_rotation_arc(Vec3::NEG_Z, -direction.direction);
        commands.entity(entity).transition(rotation, 100);

        velocity.0 = -direction.direction * move_from_target.speed;
    }
}

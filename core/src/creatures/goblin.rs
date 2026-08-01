use crate::prelude::*;

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(
            Update,
            (
                update_behavior,
                // behavior_changes.after(update_behavior),
                // attack,
                retry_move_to_target,
                // update_goblin_animtation.after(update_behavior),
            )
                .run_if(in_state(game_state)),
        );
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component, Default)]
struct GoblinBehavior {
    current: GoblinBehaviorState,
    last: GoblinBehaviorState,
    /// This is used to let goblins finish their attack before considering the next state transition
    action_interruptable: bool,
}

impl GoblinBehavior {
    fn new(state: GoblinBehaviorState) -> Self {
        Self {
            current: state,
            last: state,
            action_interruptable: true,
        }
    }
}

#[derive(Debug, Default, Reflect, PartialEq, Clone, Copy)]
enum GoblinBehaviorState {
    #[default]
    Idle,
    MovingForwards,
    MovingBackwards,
    Attacking,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[component(on_add = <Goblin as IsCreature>::on_add)]
pub struct Goblin;

impl IsCreature for Goblin {
    const NAME: &str = "Goblin";
    const MAX_HEALTH: f32 = 30.;
    const GRAVITY_SCALE: f32 = 10.;
    fn collider() -> Collider {
        Collider::cuboid(0.7, 1.225, 0.21)
    }
    fn bundle() -> impl Bundle {
        (
            CreatureLookForTarget {
                search_radius: 20.,
                search_requires_los: true,
                max_follow_distance: 25.,
                entity_filter: GameLayer::PLAYER | GameLayer::GOBLIN,
            },
            GoblinBehavior::new(GoblinBehaviorState::Idle),
        )
    }
    fn on_add_hook(mut world: DeferredWorld, this: Entity) {
        world.commands().entity(this).observe(
            |event: On<CreatureTargetUnreachable>, mut commands: Commands| {
                let entity = event.event_target();
                commands.entity(entity).insert(RetryMoveToTarget(Timer::new(
                    Duration::from_millis(1000),
                    TimerMode::Once,
                )));
            },
        );
    }
}

const GOBLIN_SPEED: f32 = 4.;
const GOBLIN_ATTACK_RANGE_MIN: f32 = 1.5;
const GOBLIN_ATTACK_RANGE_MAX: f32 = 1.7;

fn update_behavior(
    query: Query<(
        &mut GoblinBehavior,
        Option<&CreatureTarget>,
        Option<&RetryMoveToTarget>,
        &Transform,
    )>,
    targets: Query<&Transform>,
) {
    for (mut behavior, target_maybe, retry_move_maybe, transform) in query {
        if !behavior.action_interruptable {
            continue;
        }
        let state = if retry_move_maybe.is_none()
            && let Some(target) = target_maybe
            && let Ok(target_transform) = targets.get(**target)
        {
            let distance = transform.translation.distance(target_transform.translation);
            match distance {
                d if d > GOBLIN_ATTACK_RANGE_MAX => GoblinBehaviorState::MovingForwards,
                d if d < GOBLIN_ATTACK_RANGE_MIN => GoblinBehaviorState::MovingBackwards,
                _ => {
                    if behavior.action_interruptable {
                        behavior.action_interruptable = false;
                    }
                    GoblinBehaviorState::Attacking
                }
            }
        } else {
            GoblinBehaviorState::Idle
        };
        if behavior.current != state {
            behavior.last = behavior.current;
            behavior.current = state;
        }
    }
}

// fn behavior_changes(
//     mut commands: Commands,
//     query: Query<
//         (
//             Entity,
//             &GoblinBehavior,
//             &WeaponColliderHandle,
//             &mut LinearVelocity,
//         ),
//         Changed<GoblinBehavior>,
//     >,
// ) {
//     for (entity, behavior, collider_handle, mut velocity) in query {
//         if behavior.last == behavior.current {
//             continue;
//         }
//         match behavior.last {
//             GoblinBehaviorState::Attacking => {
//                 commands
//                     .entity(**collider_handle)
//                     .remove::<(Collider, Sensor)>();
//             }
//             GoblinBehaviorState::MovingForwards => {
//                 commands
//                     .entity(entity)
//                     .remove::<(CreatureMoveTowardsTarget, CreatureNavmeshPath)>();
//                 velocity.0 = Vec3::ZERO;
//             }
//             GoblinBehaviorState::MovingBackwards => {
//                 commands
//                     .entity(entity)
//                     .remove::<(CreatureMoveAwayFromTarget, CreatureDirectionToTarget)>();
//                 velocity.0 = Vec3::ZERO;
//             }
//             GoblinBehaviorState::Idle => {}
//         }
//         match behavior.current {
//             GoblinBehaviorState::Attacking => {
//                 commands.entity(**collider_handle).try_insert((
//                     Collider::cuboid(0.5, 5., 0.3),
//                     Sensor,
//                     DamageSource::new(entity, 5.),
//                 ));
//             }
//             GoblinBehaviorState::MovingForwards => {
//                 commands
//                     .entity(entity)
//                     .try_insert(CreatureMoveTowardsTarget {
//                         speed: GOBLIN_SPEED,
//                     });
//             }
//             GoblinBehaviorState::MovingBackwards => {
//                 commands
//                     .entity(entity)
//                     .try_insert(CreatureMoveAwayFromTarget {
//                         speed: GOBLIN_SPEED,
//                     });
//             }
//             GoblinBehaviorState::Idle => {}
//         }
//     }
// }

#[derive(Component, Reflect, Deref, DerefMut)]
#[reflect(Component)]
struct RetryMoveToTarget(Timer);

fn retry_move_to_target(
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(Entity, &mut RetryMoveToTarget)>,
) {
    for (entity, mut timer) in query {
        timer.tick(time.delta());
        if timer.just_finished() {
            debug!("Retrying move to target");
            commands
                .entity(entity)
                .remove::<RetryMoveToTarget>()
                .try_insert(CreatureMoveTowardsTarget {
                    speed: GOBLIN_SPEED,
                });
        }
    }
}

// fn attack(
//     mut commands: Commands,
//     query: Query<(
//         Entity,
//         &mut GoblinBehavior,
//         &GltfAnimationTarget,
//         &Transform,
//         Option<&CreatureTarget>,
//         &WeaponColliderHandle,
//     )>,
//     mut damage_sources: Query<&mut DamageSource>,
//     players: Query<&AnimationPlayer>,
//     targets: Query<&Transform>,
// ) {
//     for (entity, mut behavior, animation_target, transform, target_maybe, collider_entity) in query
//     {
//         if behavior.current != GoblinBehaviorState::Attacking {
//             continue;
//         }
//         let Some(target) = target_maybe else {
//             behavior.action_interruptable = true;
//             continue;
//         };
//         if let Ok(player) = players.get(**animation_target)
//             && player.all_finished()
//             && let Ok(target_transform) = targets.get(**target)
//         {
//             let rotation = Quat::from_rotation_arc(
//                 Vec3::NEG_Z,
//                 (transform.translation - target_transform.translation)
//                     .with_y(0.)
//                     .normalize_or(Vec3::NEG_Z),
//             );
//             commands.entity(entity).transition(rotation, 100);
//
//             // `behavior.current` is already `Attacking`, so by setting `last` to that also we
//             // ensure that the `behavior_changes` system does nothing while the
//             // `update_goblin_animtation` will restart the animation.
//             behavior.last = GoblinBehaviorState::Attacking;
//
//             behavior.action_interruptable = true;
//
//             if let Ok(mut source) = damage_sources.get_mut(**collider_entity) {
//                 source.ignore = EntityHashSet::new();
//             }
//         }
//     }
// }
//
// fn update_goblin_animtation(
//     handles: Res<GoblinHandles>,
//     query: Query<
//         (&GoblinBehavior, &GltfAnimationTarget),
//         Or<(Changed<GoblinBehavior>, Added<GltfAnimationTarget>)>,
//     >,
//     mut players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
// ) {
//     for (behavior, target) in query {
//         if let Ok((mut player, mut transitions)) = players.get_mut(**target) {
//             let (animation, speed, repeat) = match behavior.current {
//                 GoblinBehaviorState::Idle => (handles.idle, 0.1, true),
//                 GoblinBehaviorState::MovingForwards => (handles.forwards, 1., true),
//                 GoblinBehaviorState::MovingBackwards => (handles.forwards, -1., true),
//                 GoblinBehaviorState::Attacking => (handles.attack_slash, 1., false),
//             };
//             let active = transitions
//                 .play(&mut player, animation, Duration::from_millis(100))
//                 .set_speed(speed);
//             if repeat {
//                 active.repeat();
//             }
//         }
//     }
// }

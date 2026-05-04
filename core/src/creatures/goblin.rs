use crate::prelude::*;

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.load_assets_with(|asset: GltfLoadingHandle<GoblinHandles>, world| {
            let gltfs = world.resource::<Assets<Gltf>>();
            let gltf = gltfs.get(&asset.handle).unwrap();

            #[cfg(debug_assertions)]
            info!("Goblin animations:\n{:#?}", gltf.named_animations.keys());

            let (graph, clips) = match gltf.get_animations(|get| {
                get("Idle")?;
                get("AttackSlash")?;

                Ok(())
            }) {
                Ok(v) => v,
                Err(e) => panic!("{e}"),
            };

            let scene = gltf
                .default_scene
                .clone()
                .expect("No default scene in the gltf");
            let graph_handle = world.resource_mut::<Assets<AnimationGraph>>().add(graph);

            let mut scene_assets = world.resource_mut::<Assets<Scene>>();
            let scene_world = &mut scene_assets.get_mut(&scene).unwrap().world;
            let animation_players: Vec<_> = scene_world
                .query_filtered::<Entity, With<AnimationPlayer>>()
                .iter(scene_world)
                .collect();

            #[cfg(debug_assertions)]
            if animation_players.len() != 1 {
                warn!(
                    "The Player gltf has {} AnimationPlayers, expected exactly one",
                    animation_players.len()
                );
            }

            scene_world
                .commands()
                .entity(
                    *animation_players
                        .first()
                        .expect("There should be an AnimationPlayer"),
                )
                .insert((
                    AnimationTransitions::new(),
                    AnimationGraphHandle(graph_handle),
                ));
            scene_world.flush();

            GoblinHandles {
                model: scene,
                idle: clips["Idle"],
                attack_slash: clips["AttackSlash"],
            }
        });

        app.add_message::<SpawnGoblin>();

        app.add_systems(
            Update,
            spawn_goblins.run_if(in_state(game_state).and(on_message::<SpawnGoblin>)),
        );

        app.add_systems(
            Update,
            (reattack_or_move, update_goblin_animtation).run_if(in_state(game_state)),
        );
    }
}

#[derive(Resource, TypePath)]
struct GoblinHandles {
    model: Handle<Scene>,
    idle: AnimationNodeIndex,
    attack_slash: AnimationNodeIndex,
}

#[derive(Component, Debug, Default, Reflect, PartialEq)]
#[reflect(Component, Default)]
enum GoblinBehavior {
    #[default]
    Idle,
    Moving,
    Attacking,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Goblin;

impl GltfAssetPath for GoblinHandles {
    const PATH: &'static str = "models/Goblin.glb";
}

#[derive(Message, Debug)]
pub struct SpawnGoblin {
    pub position: Vec3,
}

const GOBLIN_SPEED: f32 = 4.;
const GOBLIN_ATTACK_RANGE_MIN: f32 = 1.5;
const GOBLIN_ATTACK_RANGE_MAX: f32 = 1.8;

fn spawn_goblins(
    handle: Res<GoblinHandles>,
    mut spawns: MessageReader<SpawnGoblin>,
    mut commands: Commands,
) {
    for spawn in spawns.read() {
        commands
            .spawn((
                Goblin,
                CreatureBundle {
                    name: Name::new("Goblin"),
                    health: Health::new(30.),
                    scene: SceneRoot(handle.model.clone()),
                    transform: Transform::from_translation(spawn.position),
                    collider: Collider::cuboid(0.7, 1.225, 0.21),
                    layer: CollisionLayers::new(
                        GameLayer::GOBLIN,
                        GameLayer::DEFAULT | GameLayer::ENVIRONMENT | GameLayer::DAMAGE_SOURCE,
                    ),
                    ..Default::default()
                },
                ShowHealthBar::default(),
                CreatureLookForTarget {
                    search_radius: 10.,
                    search_requires_los: true,
                    max_follow_distance: 20.,
                    entity_filter: GameLayer::PLAYER | GameLayer::GOBLIN,
                },
                CreatureMoveTowardsTarget {
                    target_gap: GOBLIN_ATTACK_RANGE_MIN,
                    speed: GOBLIN_SPEED,
                },
                GoblinBehavior::Idle,
            ))
            .observe(attack_on_target_reached)
            .observe(on_ready_insert_child_pointer::<GltfAnimationTarget>)
            .observe(on_ready_insert_child_pointer::<WeaponSocketHandle>);
    }
}

fn attack_on_target_reached(
    event: On<CreatureReachedTarget>,
    mut commands: Commands,
    mut query: Query<(&mut GoblinBehavior, &WeaponColliderHandle)>,
) {
    let Ok((mut behavior, collider_handle)) = query.get_mut(event.event_target()) else {
        return;
    };
    *behavior = GoblinBehavior::Attacking;
    commands
        .entity(**collider_handle)
        .insert(Collider::cuboid(0.5, 5., 0.3));
}

fn reattack_or_move(
    mut commands: Commands,
    query: Query<(
        Entity,
        &mut GoblinBehavior,
        &GltfAnimationTarget,
        &Transform,
        Option<&CreatureTarget>,
    )>,
    players: Query<&AnimationPlayer>,
    targets: Query<&Transform>,
) {
    for (entity, mut behavior, animation_target, transform, target_maybe) in query {
        if *behavior == GoblinBehavior::Attacking
            && let Ok(player) = players.get(**animation_target)
            && player.all_finished()
        {
            if let Some(target) = target_maybe
                && let Ok(target_transform) = targets.get(**target)
            {
                if transform.translation.distance(target_transform.translation)
                    > GOBLIN_ATTACK_RANGE_MAX
                {
                    if let Ok(mut entity_cmds) = commands.get_entity(entity) {
                        entity_cmds.insert(CreatureMoveTowardsTarget {
                            target_gap: GOBLIN_ATTACK_RANGE_MIN,
                            speed: GOBLIN_SPEED,
                        });
                        *behavior = GoblinBehavior::Moving;
                    }
                } else {
                    let rotation = Quat::from_rotation_arc(
                        Vec3::NEG_Z,
                        (transform.translation - target_transform.translation)
                            .with_y(0.)
                            .normalize(),
                    );
                    commands.entity(entity).transition(rotation, 100);
                    *behavior = GoblinBehavior::Attacking;
                }
            } else {
                *behavior = GoblinBehavior::Idle;
            }
        }
    }
}

fn update_goblin_animtation(
    handles: Res<GoblinHandles>,
    query: Query<
        (&GoblinBehavior, &GltfAnimationTarget),
        Or<(Changed<GoblinBehavior>, Added<GltfAnimationTarget>)>,
    >,
    mut players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    for (behavior, target) in query {
        if let Ok((mut player, mut transitions)) = players.get_mut(**target) {
            let (animation, speed, repeat) = match behavior {
                GoblinBehavior::Idle => (handles.idle, 0.1, true),
                GoblinBehavior::Moving => (handles.idle, 1., true),
                GoblinBehavior::Attacking => (handles.attack_slash, 1., false),
            };
            let active = transitions
                .play(&mut player, animation, Duration::from_millis(100))
                .set_speed(speed);
            if repeat {
                active.repeat();
            }
        }
    }
}

use crate::{prelude::*, weapon::WeaponSocketHandle};

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
            update_goblin_animtation.run_if(in_state(game_state)),
        );
    }
}

#[derive(Resource, TypePath)]
pub(super) struct GoblinHandles {
    model: Handle<Scene>,
    idle: AnimationNodeIndex,
    pub attack_slash: AnimationNodeIndex,
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component, Default)]
enum GoblinBehavior {
    #[default]
    Idle,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub(super) struct Goblin;

impl GltfAssetPath for GoblinHandles {
    const PATH: &'static str = "models/Goblin.glb";
}

#[derive(Message, Debug)]
pub struct SpawnGoblin {
    pub position: Vec3,
}

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
                    health: Health(30.),
                    scene: SceneRoot(handle.model.clone()),
                    transform: Transform::from_translation(spawn.position),
                    collider: Collider::cuboid(0.7, 1.225, 0.21),
                    ..Default::default()
                },
                super::ai::CreatureAiState::default(),
                super::ai::CreatureThing {
                    detection_range: 10.,
                    max_aggro_range: 20.,
                    target_distance: 2.,
                    speed: 3.,
                    hostile_towards: super::ai::CreatureFaction::PLAYER,
                },
                GoblinBehavior::Idle,
            ))
            .observe(on_ready_insert_child_pointer::<GltfAnimationTarget>)
            .observe(on_ready_insert_child_pointer::<WeaponSocketHandle>);
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
            match behavior {
                GoblinBehavior::Idle => {
                    transitions
                        .play(&mut player, handles.idle, Duration::from_millis(100))
                        .set_speed(0.1)
                        .repeat();
                }
            }
        }
    }
}

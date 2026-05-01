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
struct GoblinHandles {
    model: Handle<Scene>,
    idle: AnimationNodeIndex,
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component, Default)]
enum GoblinBehavior {
    #[default]
    Idle,
}

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
                Name::new("Goblin"),
                GoblinBehavior::Idle,
                Health(30.),
                SceneRoot(handle.model.clone()),
                Transform::from_translation(spawn.position),
                RigidBody::Dynamic,
                LockedAxes::ROTATION_LOCKED,
                Collider::cuboid(0.7, 1.225, 0.21),
                GravityScale(10.),
                CollisionLayers::new(CollisionLayer::Creature, CollisionLayer::all_bits()),
            ))
            .observe(on_ready_insert_child_pointer::<GltfAnimationTarget>);
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

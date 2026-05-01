use crate::{character_controller::actions::CharacterActions, prelude::*};

pub mod actions;
mod weapon;

pub fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_plugins((actions::plugin(game_state), weapon::plugin));

        app.load_assets_with(|handle: GltfLoadingHandle<PlayerCharacterHandle>, world| {
            let gltf = handle.get_gltf(world);

            #[cfg(debug_assertions)]
            info!("Player animations:\n{:#?}", gltf.named_animations.keys());

            let (graph, clips) = match gltf.get_animations(|get| {
                get("Idle")?;
                get("Running")?;
                get("Jumping")?;
                get("Falling")?;
                get("Attack")?;
                get("AttackSwingBottom")?;

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

            PlayerCharacterHandle {
                scene,
                idle: clips["Idle"],
                running: clips["Running"],
                jumping: clips["Jumping"],
                falling: clips["Falling"],
                attack: clips["Attack"],
                attack_bottom: clips["AttackSwingBottom"],
            }
        });

        app.add_message::<SpawnPlayer>();
        app.add_systems(Update, spawn_player.run_if(on_message::<SpawnPlayer>));
    }
}

#[derive(Resource, TypePath)]
pub struct PlayerCharacterHandle {
    scene: Handle<Scene>,
    idle: AnimationNodeIndex,
    running: AnimationNodeIndex,
    jumping: AnimationNodeIndex,
    falling: AnimationNodeIndex,
    attack: AnimationNodeIndex,
    attack_bottom: AnimationNodeIndex,
}

impl GltfAssetPath for PlayerCharacterHandle {
    const PATH: &'static str = "models/Player.glb";
}

#[derive(Component)]
pub struct MainCharacter;

#[derive(Message, Debug)]
pub struct SpawnPlayer {
    pub position: Vec3,
}

fn spawn_player(
    mut reader: MessageReader<SpawnPlayer>,
    mut commands: Commands,
    player_handle: Res<PlayerCharacterHandle>,
) {
    for spawn in reader.read() {
        commands
            .spawn((
                Name::new("Player"),
                SceneRoot(player_handle.scene.clone()),
                MainCharacter,
                CharacterActions::default(),
                Transform::from_translation(spawn.position),
                RigidBody::Dynamic,
                LockedAxes::ROTATION_LOCKED,
                Collider::cuboid(0.5, 1.94, 0.2),
                GravityScale(10.),
                CollisionLayers::new(
                    CollisionLayer::Creature,
                    CollisionLayer::all_bits() & !CollisionLayer::Creature.to_bits(),
                ),
            ))
            .observe(on_ready_insert_child_pointer::<GltfAnimationTarget>)
            .observe(on_ready_insert_child_pointer::<weapon::WeaponSocketHandle>);
    }
}

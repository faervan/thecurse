use crate::{
    CameraControllerAnchor, character_controller::actions::CharacterActions, prelude::*,
    utils::gltf_instance_hooks::on_ready_insert_animation_target,
};

mod actions;

pub fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_plugins(actions::plugin(game_state));

        app.load_assets_with(|handle: GltfLoadingHandle<PlayerCharacterHandle>, world| {
            let gltf = handle.get_gltf(world);

            #[cfg(debug_assertions)]
            info!("Player animations:\n{:#?}", gltf.named_animations.keys());

            let (graph, clips) = match gltf.get_animations(|get| {
                get("Idle")?;
                get("Running")?;
                get("Jumping")?;
                get("Falling")?;

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
}

impl GltfAssetPath for PlayerCharacterHandle {
    const PATH: &'static str = "models/Player.glb";
}

#[derive(Message, Debug)]
pub struct SpawnPlayer {
    pub position: Vec3,
}

fn spawn_player(
    mut reader: MessageReader<SpawnPlayer>,
    mut commands: Commands,
    ankor: Single<Entity, With<CameraControllerAnchor>>,
    player_handle: Res<PlayerCharacterHandle>,
) {
    for spawn in reader.read() {
        let id = commands
            .spawn((
                Name::new("Player"),
                SceneRoot(player_handle.scene.clone()),
                CharacterActions::default(),
                Transform::from_translation(spawn.position),
                RigidBody::Dynamic,
                LockedAxes::ROTATION_LOCKED,
                Collider::cuboid(0.5, 2., 0.2),
                GravityScale(10.),
            ))
            .observe(on_ready_insert_animation_target)
            .id();

        commands.entity(*ankor).insert(ChildOf(id));
    }
}

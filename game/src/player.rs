use bevy::{ecs::observer::ObservedBy, scene::SceneInstanceReady};
use thecurse_core::{
    CameraControllerAnchor,
    utils::{GltfAssetPath, GltfLoadingHandle},
};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.load_assets::<GltfLoadingHandle<PlayerCharacterHandle>>();
    app.transform_resource_on_add(|world, handle: GltfLoadingHandle<PlayerCharacterHandle>| {
        let gltf = handle.get_gltf(world);

        #[cfg(debug_assertions)]
        info!("Player animations:\n{:#?}", gltf.named_animations.keys());

        let (graph, clips) = match gltf.get_animations(|get| {
            get("Jumping")?;

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
            .insert(AnimationGraphHandle(graph_handle));

        PlayerCharacterHandle {
            scene,
            jumping: clips["Jumping"],
        }
    });

    app.add_systems(
        OnEnter(AppState::Game),
        spawn_player.after(thecurse_core::spawn_camera),
    );
    app.add_systems(Update, player_input);
}

#[derive(Resource, TypePath)]
struct PlayerCharacterHandle {
    scene: Handle<Scene>,
    jumping: AnimationNodeIndex,
}

impl GltfAssetPath for PlayerCharacterHandle {
    const PATH: &'static str = "models/Player.glb";
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct PlayerAnimationTarget(Entity);

fn spawn_player(
    mut commands: Commands,
    ankor: Single<Entity, With<CameraControllerAnchor>>,
    player_handle: Res<PlayerCharacterHandle>,
) {
    let id = commands
        .spawn((
            Name::new("Player"),
            SceneRoot(player_handle.scene.clone()),
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            Collider::cuboid(0.5, 2., 0.2),
            GravityScale(0.),
        ))
        .observe(
            |event: On<SceneInstanceReady>,
             mut commands: Commands,
             query: Query<(Option<&Children>, Has<AnimationPlayer>)>| {
                let mut current = vec![event.entity];
                'outer: loop {
                    for entity in std::mem::take(&mut current) {
                        if let Ok((children_maybe, has_player)) = query.get(entity) {
                            if has_player {
                                commands
                                    .entity(event.entity)
                                    .insert(PlayerAnimationTarget(entity));
                                break 'outer;
                            }
                            if let Some(children) = children_maybe {
                                current.extend(children);
                            }
                        }
                    }
                }
                // Remove this observer
                commands.entity(event.entity).remove::<ObservedBy>();
            },
        )
        .id();

    commands.entity(*ankor).insert(ChildOf(id));
}

fn player_input(input: Res<ButtonInput<KeyCode>>) {}

use crate::{player::PlayerCharacterHandle, prelude::*};

pub(super) fn load_player_assets(
    handle: GltfLoadingHandle<PlayerCharacterHandle>,
    world: &mut World,
) -> PlayerCharacterHandle {
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

    let mut scene_assets = world.resource_mut::<Assets<WorldAsset>>();
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
}

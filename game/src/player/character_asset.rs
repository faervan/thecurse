use crate::{
    assets::gltf_loading::{GltfAssetPath, GltfLoadingHandle},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.load_assets_with(load_player_assets);
}

#[derive(Resource, TypePath)]
pub(super) struct PlayerCharacterHandle {
    _scene: Handle<WorldAsset>,
    _idle: AnimationNodeIndex,
    _running: AnimationNodeIndex,
    _jumping: AnimationNodeIndex,
    _falling: AnimationNodeIndex,
    _attack: AnimationNodeIndex,
    _attack_bottom: AnimationNodeIndex,
}

impl GltfAssetPath for PlayerCharacterHandle {
    const PATH: &'static str = "models/Player.glb";
}

fn load_player_assets(
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
        _scene: scene,
        _idle: clips["Idle"],
        _running: clips["Running"],
        _jumping: clips["Jumping"],
        _falling: clips["Falling"],
        _attack: clips["Attack"],
        _attack_bottom: clips["AttackSwingBottom"],
    }
}

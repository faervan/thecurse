use crate::{
    assets::gltf_loading::{GltfAssetPath, GltfLoadingHandle},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.load_assets_with(load_player_assets);
    app.add_systems(Update, debug_bones);
}

#[derive(Resource, TypePath)]
pub(super) struct PlayerCharacterHandle {
    _scene: Handle<WorldAsset>,
    pub idle: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
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
        get("Run")?;

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
        idle: clips["Idle"],
        run: clips["Run"],
    }
}

//
// See <https://bevy.org/examples/animation/animation-masks/>
//
const MASK_GROUP_HEAD: u32 = 0;
const MASK_GROUP_LEFT_ARM: u32 = 1;
const MASK_GROUP_RIGHT_ARM: u32 = 2;
const MASK_GROUP_LEGS: u32 = 3;

fn debug_bones(
    bones: Query<(Entity, &Name, Option<&Children>), Added<bevy::animation::AnimationTargetId>>,
) {
    if bones.count() != 0 {
        debug!("bones: {}", bones.count());
    }
    for (entity, name, maybe_children) in bones {
        if maybe_children.is_none() {
            debug!("{name} is end of chain");
        }
    }
}

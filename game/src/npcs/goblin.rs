use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.load_assets_with(|asset: GltfLoadingHandle<GoblinHandles>, world| {
        let gltfs = world.resource::<Assets<Gltf>>();
        let gltf = gltfs.get(&asset.handle).unwrap();

        #[cfg(debug_assertions)]
        info!("Goblin animations:\n{:#?}", gltf.named_animations.keys());

        let (graph, clips) = match gltf.get_animations(|get| {
            get("Idle")?;
            get("WalkForwards")?;
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

        GoblinHandles {
            scene,
            idle: clips["Idle"],
            forwards: clips["WalkForwards"],
            attack_slash: clips["AttackSlash"],
        }
    });

    app.add_observer(on_goblin_spawn);
}

#[derive(Resource, TypePath)]
struct GoblinHandles {
    scene: Handle<WorldAsset>,
    idle: AnimationNodeIndex,
    forwards: AnimationNodeIndex,
    attack_slash: AnimationNodeIndex,
}

impl GltfAssetPath for GoblinHandles {
    const PATH: &'static str = "models/Goblin.glb";
}

fn on_goblin_spawn(event: On<Add, Goblin>, goblin: Res<GoblinHandles>, mut commands: Commands) {
    commands
        .entity(event.entity)
        .try_insert((
            WorldAssetRoot(goblin.scene.clone()),
            ShowHealthBar::default(),
        ))
        .observe(on_ready_insert_child_pointer::<GltfAnimationTarget>)
        .observe(on_ready_insert_child_pointer::<WeaponSocketHandle>);
}

use std::sync::Arc;

use crate::prelude::*;

pub fn setup(mut commands: Commands) {
    commands.spawn(RasterizedGridObj);
    commands.spawn((PointLightObj, Transform::from_xyz(5., 3., 3.)));
    commands.spawn((RockObj, Transform::from_xyz(10., 2.5, 10.)));
}

#[derive(Resource, Default, Clone)]
pub struct SerializedScene {
    pub notify: Arc<event_listener::Event>,
    pub world: Arc<smol::lock::RwLock<String>>,
}

pub fn scene_requested(scene: Res<SerializedScene>) -> bool {
    scene.notify.total_listeners() > 0
}

pub fn publish_scene(scene: In<String>, scene_res: Res<SerializedScene>) {
    *scene_res.world.write_arc_blocking() = scene.0;
    scene_res.notify.notify(usize::MAX);
}

pub fn serialize_scene(world: &World, query: Query<Entity, With<GameStateEntity>>) -> String {
    debug!("running serialize_scene");
    let type_registry = world.resource::<AppTypeRegistry>().read();
    let scene = DynamicWorldBuilder::from_world(world, &type_registry)
        //
        // Allowed resources
        //
        .deny_all_resources()
        //
        // Allowed components
        //
        // Creatures
        .allow_component::<Player>()
        .allow_component::<Goblin>()
        // Environment
        .allow_component::<PointLightObj>()
        .allow_component::<RasterizedGridObj>()
        .allow_component::<RockObj>()
        // Misc
        .allow_component::<Transform>()
        .allow_component::<Name>()
        .allow_component::<Health>()
        .allow_component::<ClientId>()
        //
        // Extraction
        //
        .extract_entities(query.iter())
        .extract_resources()
        .build();

    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = type_registry.read();

    scene.serialize(&type_registry).unwrap()
}

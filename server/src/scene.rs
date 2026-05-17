use crate::prelude::*;

pub fn setup(mut commands: Commands) {
    commands.spawn(RasterizedGridObj);
    commands.spawn((PointLightObj, Transform::from_xyz(5., 3., 3.)));
    commands.spawn((RockObj, Transform::from_xyz(10., 2.5, 10.)));
}

pub fn serialize_scene(world: &World, query: Query<Entity, With<GameStateEntity>>) -> String {
    let scene = DynamicSceneBuilder::from_world(world)
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
        .allow_component::<Health>()
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

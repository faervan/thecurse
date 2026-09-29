use crate::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(PlayerCharacter)]
pub struct MainCharacter;

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(PlayerCharacter)]
pub struct ScriptedPlayer;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Player)]
#[component(on_add)]
struct PlayerCharacter;

impl PlayerCharacter {
    fn on_add(mut world: DeferredWorld, hook: HookContext) {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh = meshes.add(Capsule3d::new(0.2, 1.94));

        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.1),
            ..Default::default()
        });

        world
            .commands()
            .entity(hook.entity)
            .insert((Mesh3d(mesh), MeshMaterial3d(material)));
    }
}

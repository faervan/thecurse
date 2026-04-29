use crate::{prelude::*, utils::gltf_instance_hooks::ChildEntityPointer};

pub(super) fn plugin(app: &mut App) {
    app.load_assets_with(|asset: PlayerWeaponsAsset, world: &mut World| {
        let gltfs = world.resource::<Assets<Gltf>>();
        let gltf = gltfs.get(&asset.sword).unwrap();
        PlayerWeapons {
            sword: gltf.default_scene.clone().expect("No default scene"),
        }
    });
}

#[derive(Asset, TypePath)]
struct PlayerWeaponsAsset {
    sword: Handle<Gltf>,
}

impl FromWorld for PlayerWeaponsAsset {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            sword: asset_server.load("models/Sword.glb"),
        }
    }
}

#[derive(Resource)]
struct PlayerWeapons {
    sword: Handle<Scene>,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[component(on_add)]
pub struct WeaponSocket;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct WeaponSocketHandle(Entity);

impl ChildEntityPointer for WeaponSocketHandle {
    type Target = WeaponSocket;
    fn new(entity: Entity) -> Self {
        Self(entity)
    }
}

impl WeaponSocket {
    fn on_add(mut world: DeferredWorld, hook: HookContext) {
        let sword = world.resource::<PlayerWeapons>().sword.clone();
        world
            .commands()
            .spawn((Name::new("Sword"), SceneRoot(sword), ChildOf(hook.entity)));
    }
}

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
    #[dependency]
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
pub struct WeaponSocket;

#[derive(Component, Reflect, Deref)]
#[reflect(Component)]
#[component(on_add)]
pub struct WeaponSocketHandle(Entity);

impl ChildEntityPointer for WeaponSocketHandle {
    type Target = WeaponSocket;
    fn new(entity: Entity) -> Self {
        Self(entity)
    }
}

#[derive(Component, Reflect, Deref)]
#[reflect(Component)]
pub struct WeaponColliderHandle(Entity);

impl ChildEntityPointer for WeaponColliderHandle {
    type Target = WeaponSocket;
    fn new(entity: Entity) -> Self {
        Self(entity)
    }
}

impl WeaponSocketHandle {
    fn on_add(mut world: DeferredWorld, hook: HookContext) {
        let socket_entity = world.get::<Self>(hook.entity).unwrap().0;
        let sword = world.resource::<PlayerWeapons>().sword.clone();
        let mut id = None;
        world
            .commands()
            .spawn((Name::new("Sword"), SceneRoot(sword), ChildOf(socket_entity)))
            .with_children(|p| {
                id = Some(
                    p.spawn((
                        Transform::from_xyz(0., 4., 0.),
                        Collider::cuboid(0.5, 5., 0.3),
                        Sensor,
                    ))
                    .observe(
                        |event: On<CollisionStart>, mut damage: MessageWriter<DealDamage>| {
                            debug!("Sword {} hit {}", event.collider1, event.collider2);
                            damage.write(DealDamage {
                                target: event.collider2,
                                amount: 5.,
                            });
                        },
                    )
                    .id(),
                );
            });
        world
            .commands()
            .entity(hook.entity)
            .insert(WeaponColliderHandle(id.unwrap()));
    }
}

use crate::{prelude::*, utils::gltf_instance_hooks::ChildEntityPointer};

pub(super) fn plugin(app: &mut App) {
    app.load_assets_with(
        |asset: GltfLoadingHandle<PlayerWeapons>, world: &mut World| {
            let gltfs = world.resource::<Assets<Gltf>>();
            let gltf = gltfs.get(&asset.handle).unwrap();
            PlayerWeapons {
                sword: gltf.default_scene.clone().expect("No default scene"),
            }
        },
    );
}

#[derive(Resource, TypePath)]
struct PlayerWeapons {
    sword: Handle<Scene>,
}

impl GltfAssetPath for PlayerWeapons {
    const PATH: &'static str = "models/Sword.glb";
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
                        RigidBody::Kinematic,
                        Sensor,
                        CollisionEventsEnabled,
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
        let id = id.unwrap();
        world
            .commands()
            .entity(hook.entity)
            .insert(WeaponColliderHandle(id));
    }
}

use crate::prelude::*;

trait IsEnvironmentObject: Component + Default {
    const NAME: &str;
    fn bundle() -> impl Bundle {}
    fn on_add(mut world: DeferredWorld, hook: HookContext) {
        world
            .commands()
            .entity(hook.entity)
            .try_insert_if_new((Self::bundle(), GameEntity, GameStateEntity))
            .try_insert_if_new(EnvironmentObjectBundle::<Self>::default());
    }
}

#[derive(Bundle)]
struct EnvironmentObjectBundle<E: IsEnvironmentObject> {
    name: Name,
    transform: Transform,
    e: E,
}

impl<E: IsEnvironmentObject> Default for EnvironmentObjectBundle<E> {
    fn default() -> Self {
        Self {
            name: Name::new(E::NAME),
            transform: Transform::default(),
            e: E::default(),
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[component(on_add = <Self as IsEnvironmentObject>::on_add)]
pub struct PointLightObj;

impl IsEnvironmentObject for PointLightObj {
    const NAME: &str = "Light";
    fn bundle() -> impl Bundle {
        #[cfg(feature = "game")]
        PointLight {
            intensity: 1_000_000.,
            range: 50.,
            shadows_enabled: true,
            ..Default::default()
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[component(on_add = <Self as IsEnvironmentObject>::on_add)]
pub struct RasterizedGridObj;

impl IsEnvironmentObject for RasterizedGridObj {
    const NAME: &str = "Ground";
    fn bundle() -> impl Bundle {
        children![(
            Name::new("Ground Collider"),
            PhysicsPickable,
            RigidBody::Static,
            Collider::cuboid(100., 1., 100.),
            CollisionLayers::new(GameLayer::ENVIRONMENT, GameLayer::all()),
            Transform::from_xyz(0., -0.5, 0.)
        )]
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[component(on_add = <Self as IsEnvironmentObject>::on_add)]
pub struct RockObj;

impl IsEnvironmentObject for RockObj {
    const NAME: &str = "Rock";
    fn bundle() -> impl Bundle {
        (
            Obstacle,
            PhysicsPickable,
            RigidBody::Static,
            Collider::cuboid(5., 5., 5.),
            CollisionLayers::new(GameLayer::ENVIRONMENT, GameLayer::all()),
        )
    }
}

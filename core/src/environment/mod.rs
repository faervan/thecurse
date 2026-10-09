use crate::prelude::*;

#[derive(Component, Reflect, ByteRepr, Debug, Clone, Copy)]
#[reflect(Component)]
pub enum EnvironmentObj {
    RasterizedGrid,
    Rock,
}

#[derive(SceneComponent, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
#[require(EnvironmentObj::RasterizedGrid)]
pub struct RasterizedGridCollider;

impl RasterizedGridCollider {
    fn scene() -> impl Scene {
        bsn! {
            #RasterizedGridCollider
            template_value(RigidBody::Static)
            Collider::cuboid(100., 1., 100.)
            CollisionLayers::new(GameLayer::ENVIRONMENT, GameLayer::all())
            Transform::from_xyz(0., -0.5, 0.)
        }
    }
}

#[derive(SceneComponent, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
#[require(EnvironmentObj::Rock)]
pub struct RockCollider;

impl RockCollider {
    fn scene() -> impl Scene {
        bsn! {
            #RockCollider
            template_value(RigidBody::Static)
            Collider::cuboid(5., 5., 5.)
            CollisionLayers::new(GameLayer::ENVIRONMENT, GameLayer::all())
        }
    }
}

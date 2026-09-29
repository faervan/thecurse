use crate::prelude::*;

#[derive(SceneComponent, Default, Clone)]
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

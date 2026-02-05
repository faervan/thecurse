use crate::prelude::*;

#[derive(PhysicsLayer, Clone, Copy, Default)]
pub enum CollisionLayer {
    #[default]
    Default,
    Environment,
    Creature,
    Projectile,
}

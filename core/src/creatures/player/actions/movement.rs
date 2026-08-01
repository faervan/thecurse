use crate::prelude::*;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct MovementState {
    pub direction: Vec3,
}

pub const MOVEMENT_SPEED: f32 = 10.;
pub const AERIAL_MOVEMENT_FACTOR: f32 = 0.8;

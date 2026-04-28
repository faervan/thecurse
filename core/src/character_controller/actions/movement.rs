use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Moving {
    pub direction: Vec3,
}

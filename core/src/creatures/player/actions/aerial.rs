use crate::prelude::*;

#[derive(Component, Reflect, Debug, Default, PartialEq)]
#[reflect(Component)]
pub enum AerialState {
    #[default]
    Grounded,
    Jumping,
    Falling,
}

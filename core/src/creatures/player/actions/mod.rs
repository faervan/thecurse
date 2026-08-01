use crate::prelude::*;

mod attack;
pub use attack::*;

mod movement;
pub use movement::*;

mod aerial;
pub use aerial::*;

#[derive(Bundle, Default)]
pub struct CharacterActions {
    pub aerial: AerialState,
    pub movement: MovementState,
    pub attack: AttackState,
}

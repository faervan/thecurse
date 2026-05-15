use crate::{
    player::actions::{aerial::AerialState, attack::AttackState, movement::MovementState},
    prelude::*,
};

mod aerial;
mod attack;
mod movement;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((movement::plugin, aerial::plugin, attack::plugin));

    app.add_message::<InterruptAction>();
}

#[derive(Bundle, Default)]
pub struct CharacterActions {
    pub aerial: AerialState,
    pub movement: MovementState,
    pub attack: AttackState,
}

#[derive(Message, Debug)]
pub enum InterruptAction {
    PlayerJumped,
}

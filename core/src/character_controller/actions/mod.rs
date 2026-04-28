use crate::{character_controller::actions::aerial::AerialState, prelude::*};

mod aerial;
mod movement;

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_plugins((movement::plugin, aerial::plugin(game_state)));
    }
}

#[derive(Bundle, Default)]
pub struct CharacterActions {
    pub aerial: AerialState,
}

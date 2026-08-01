use crate::prelude::*;

mod aerial;
mod attack;
mod movement;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((movement::plugin, aerial::plugin, attack::plugin));

    app.add_message::<InterruptAction>();
}

#[derive(Message, Debug)]
pub enum InterruptAction {
    PlayerJumped,
}

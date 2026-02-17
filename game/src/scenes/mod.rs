use crate::prelude::*;

mod flat;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(flat::plugin);

    app.add_sub_state::<GameScene>();
}

#[derive(SubStates, Debug, Default, Clone, Hash, PartialEq, Eq)]
#[source(AppState = AppState::Game)]
pub enum GameScene {
    #[default]
    Flat,
}

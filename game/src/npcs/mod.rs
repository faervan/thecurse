use crate::prelude::*;

mod goblin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(goblin::plugin);
}

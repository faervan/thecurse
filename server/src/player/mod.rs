use crate::prelude::*;

mod actions;

pub fn plugin(app: &mut App) {
    app.add_plugins(actions::plugin);
}

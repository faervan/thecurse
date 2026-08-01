use crate::prelude::*;

mod attack;

pub fn plugin(app: &mut App) {
    app.add_plugins(attack::plugin);
}

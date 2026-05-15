use crate::prelude::*;

pub mod health_bar;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(health_bar::plugin);
}

use crate::prelude::*;

pub mod actions;

pub fn plugin(app: &mut App) {
    app.add_plugins(actions::plugin);
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct PlayerBroadcast {
    pub movement_changed: bool,
    /// Whether the player just started moving this frame.
    pub first_movement_after_idle: bool,
}

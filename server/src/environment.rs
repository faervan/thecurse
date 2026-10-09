use dreamgame_core::environment::{RasterizedGridCollider, RockCollider};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_scene);
}

fn spawn_scene(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        @RasterizedGridCollider
    });
    commands.spawn_scene(bsn! {
        @RockCollider
        Transform::from_xyz(10., 2.5, 10.)
    });
}

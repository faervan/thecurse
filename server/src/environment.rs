use dreamgame_core::environment::RasterizedGridCollider;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_scene);
}

fn spawn_scene(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        @RasterizedGridCollider
    });
}

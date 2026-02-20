use thecurse_core::{CameraControllerPlugin, CharacterControllerPlugin, asset_plugin};

use crate::prelude::*;

mod player;
mod prelude;
mod scenes;

fn main() {
    let mut app = App::new();

    // Bevy default plugins
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "The Curse".to_string(),
                    name: Some("thecurse".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(asset_plugin()),
    );

    // Ecosystem plugins
    app.add_plugins(PhysicsPlugins::default());

    // Custom plugins
    app.add_plugins((
        thecurse_core::debug::plugin,
        CameraControllerPlugin::<AppState>::default(),
        CharacterControllerPlugin,
        player::plugin,
        scenes::plugin,
    ));

    // States
    app.init_state::<AppState>();

    app.run();
}

#[derive(States, Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
pub enum AppState {
    #[default]
    Game,
}

use thecurse_core::{CameraControllerPlugin, asset_plugin, assets::all_assets_loaded};

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
                    // TODO! This needs to be AutoVsync if not on wayland
                    present_mode: bevy::window::PresentMode::Mailbox,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(asset_plugin()),
    );

    // Custom plugins
    app.add_plugins((
        thecurse_core::default_plugins(AppState::Game),
        thecurse_core::debug::plugin,
        CameraControllerPlugin::new(AppState::Game),
        player::plugin,
        scenes::plugin,
    ));

    // States
    app.init_state::<AppState>();

    app.add_systems(
        Update,
        set_state_game.run_if(in_state(AppState::Loading).and(all_assets_loaded)),
    );

    app.run();
}

#[derive(States, Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
pub enum AppState {
    #[default]
    Loading,
    Game,
}

fn set_state_game(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Game);
}

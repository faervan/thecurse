use thecurse_core::{CameraControllerPlugin, asset_plugin, assets::all_assets_loaded};

use crate::prelude::*;

mod menu;
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
        menu::plugin,
        player::plugin,
        scenes::plugin,
    ));

    // States
    app.init_state::<AppState>();

    app.add_systems(
        Update,
        (
            set_state_menu.run_if(in_state(AppState::Loading).and(all_assets_loaded)),
            set_state_menu
                .run_if(in_state(AppState::Game).and(input_just_pressed(KeyCode::Escape))),
        ),
    );

    app.run();
}

#[derive(States, Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
pub enum AppState {
    #[default]
    Loading,
    Menu,
    Game,
}

fn set_state_menu(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Menu);
}

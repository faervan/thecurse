use bevy::dev_tools::render_debug::RenderDebugOverlayPlugin;
use thecurse_core::{asset_plugin, assets::all_assets_loaded};

use crate::prelude::*;

use clap::Parser;

mod camera;
mod console;
mod debug;
mod environment;
mod hud;
mod menu;
mod networking;
mod npcs;
mod player;
mod prelude;
mod scenes;
mod utils;
mod weapon;

#[derive(Parser, Debug, Resource, Reflect)]
#[command(version, about)]
/// Game binary of "The Curse".
pub struct GameSettings {
    #[arg(short, long, default_value_t = 7188)]
    /// UDP port to connect to.
    port_udp: u16,

    #[arg(short = 'P', long, default_value_t = 7189)]
    /// TCP port to connect to.
    port_tcp: u16,

    #[arg(short, long)]
    #[cfg_attr(debug_assertions, arg(default_value = "0.0.0.0"))]
    #[cfg_attr(not(debug_assertions), arg(default_value = "72.61.104.16"))]
    /// Server address to connect to.
    addr: String,
}

fn main() {
    let settings = GameSettings::parse();

    let mut app = App::new();

    app.insert_resource(settings);

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
            .set(asset_plugin())
            .disable::<RenderDebugOverlayPlugin>(),
    );

    // Bevy ecosystem plugins
    app.add_plugins(bevy_skein::SkeinPlugin::default());

    // Custom plugins
    app.add_plugins((
        thecurse_core::default_plugins(AppState::Game),
        debug::plugin,
        console::plugin,
        camera::CameraControllerPlugin::new(AppState::Game),
        menu::plugin,
        utils::plugin,
        networking::plugin,
        hud::plugin,
        scenes::plugin,
        environment::plugin,
        weapon::plugin,
        player::plugin,
        npcs::plugin,
    ));

    // States
    app.init_state::<AppState>();

    app.add_systems(
        Update,
        (
            set_state_menu.run_if(in_state(AppState::Loading).and_then(all_assets_loaded)),
            return_to_menu
                .run_if(in_state(AppState::Game).and_then(input_just_pressed(KeyCode::Escape))),
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

fn return_to_menu(mut next_state: ResMut<NextState<AppState>>, mut udp: ResMut<Udp>) {
    next_state.set(AppState::Menu);
    udp.disconnect();
}

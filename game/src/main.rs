use dreamgame_core::asset_plugin;

use crate::prelude::*;

mod asset_loader;
mod camera;
mod debug;
mod environment;
mod menu;
mod networking;
mod player;
mod prelude;
mod settings;
mod state;

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
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(asset_plugin()),
    );

    // Bevy ecosystem plugins
    app.add_plugins((
        bevy_skein::SkeinPlugin::default(),
        PhysicsPlugins::default(),
    ));

    // Custom plugins
    app.add_plugins((
        state::plugin,
        menu::plugin,
        asset_loader::plugin,
        camera::plugin,
        debug::plugin,
        networking::plugin,
    ));
    app.run();
}

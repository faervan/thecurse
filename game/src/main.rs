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

    app.add_systems(OnEnter(AppState::Game), demo);

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

fn demo(mut goblin_spawner: MessageWriter<SpawnGoblin>) {
    goblin_spawner.write(SpawnGoblin {
        position: Vec3::new(0., 1., 5.),
    });
    goblin_spawner.write(SpawnGoblin {
        position: Vec3::new(5., 1., 5.),
    });
    goblin_spawner.write(SpawnGoblin {
        position: Vec3::new(10., 1., 8.),
    });
    goblin_spawner.write(SpawnGoblin {
        position: Vec3::new(5., 1., 25.),
    });
}

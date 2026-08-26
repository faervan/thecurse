use crate::{asset_loader::all_assets_loaded, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.init_state::<AppState>();

    // ---------------- Loading -> Menu ----------------
    app.add_systems(
        Update,
        set_state_menu.run_if(in_state(AppState::Loading).and_then(all_assets_loaded)),
    );

    // ---------------- Menu -> Game ----------------
    app.add_systems(
        Update,
        (|mut next_state: ResMut<NextState<AppState>>| next_state.set(AppState::Game)).run_if(
            in_state(AppState::Menu).and_then(
                input_just_pressed(KeyCode::KeyG).or_else(input_just_pressed(KeyCode::KeyP)),
            ),
        ),
    );
    app.add_systems(
        Update,
        (|mut exit: MessageWriter<AppExit>| {
            exit.write(AppExit::Success);
        })
        .run_if(in_state(AppState::Menu).and_then(input_just_pressed(KeyCode::KeyQ))),
    );

    // ---------------- Game -> Menu ----------------
    app.add_systems(
        Update,
        return_to_menu
            .run_if(in_state(AppState::Game).and_then(input_just_pressed(KeyCode::Escape))),
    );
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

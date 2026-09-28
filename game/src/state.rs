use crate::{asset_loader::all_assets_loaded, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.init_state::<AppState>();
    app.add_sub_state::<Connected>();

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

    // ---------------- Connected state updates ----------------
    app.add_systems(Update, set_connected_state.run_if(in_state(AppState::Game)));
}

#[derive(States, Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
pub enum AppState {
    #[default]
    Loading,
    Menu,
    Game,
}

#[derive(SubStates, Clone, Copy, Default, Debug, Hash, PartialEq, Eq)]
#[source(AppState = AppState::Game)]
pub struct Connected(pub bool);

fn set_state_menu(mut next_state: ResMut<NextState<AppState>>) {
    next_state.set(AppState::Menu);
}

fn return_to_menu(mut next_state: ResMut<NextState<AppState>>, mut udp: ResMut<Udp>) {
    next_state.set(AppState::Menu);
    udp.disconnect();
}

fn set_connected_state(
    mut next_state: ResMut<NextState<Connected>>,
    state: Res<State<Connected>>,
    udp: Res<Udp>,
) {
    let state_is_connected = state.get().0;
    if udp.state().is_connected() && !state_is_connected {
        next_state.set(Connected(true));
    } else if state_is_connected {
        next_state.set(Connected(false));
    }
}

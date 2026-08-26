use crate::{asset_loader::all_assets_loaded, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.init_state::<AppState>();

    app.add_systems(
        Update,
        (
            set_state_menu.run_if(in_state(AppState::Loading).and_then(all_assets_loaded)),
            return_to_menu
                .run_if(in_state(AppState::Game).and_then(input_just_pressed(KeyCode::Escape))),
        ),
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

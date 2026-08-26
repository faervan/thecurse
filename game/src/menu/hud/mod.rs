use crate::{
    menu::{root_node, text},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Game), build_hud);

    app.add_systems(Update, update_state.run_if(in_state(AppState::Game)));
}

#[derive(Component)]
struct UdpState;

fn build_hud(mut commands: Commands) {
    commands.spawn((Camera2d, DespawnOnExit(AppState::Game)));

    commands.spawn((
        root_node(
            "HUD Root",
            JustifyContent::Center,
            AlignContent::Start,
            FlexDirection::Column,
            px(8),
            UiRect::horizontal(percent(10)),
        ),
        DespawnOnExit(AppState::Game),
        children![text(
            "State: Disconnected\nLast seen: 0ms ago",
            26.,
            UdpState
        )],
    ));
}

fn update_state(udp: Res<Udp>, mut text: Single<&mut Text, With<UdpState>>) {
    text.0 = udp.debug_state();
}

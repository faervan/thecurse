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
            JustifyContent::Start,
            AlignContent::End,
            FlexDirection::Column,
            px(8),
            UiRect::axes(px(20), px(10)),
        ),
        DespawnOnExit(AppState::Game),
        children![(
            text("State: Disconnected\nLast seen: 0ms ago", 26.,),
            UdpState,
            Node {
                right: Val::ZERO,
                align_self: AlignSelf::End,
                ..Default::default()
            }
        )],
    ));
}

fn update_state(
    udp: Res<Udp>,
    mut text: Single<&mut Text, With<UdpState>>,
    time: Res<Time>,
    mut timer: Local<Timer>,
) {
    if timer.duration().is_zero() {
        *timer = Timer::new(Duration::from_millis(100), TimerMode::Repeating);
    }
    timer.tick(time.delta());
    if timer.just_finished() {
        text.0 = udp.debug_state();
    }
}

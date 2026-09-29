use crate::{menu::text, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Game), build_hud);

    app.add_systems(Update, update_state.run_if(in_state(AppState::Game)));
}

#[derive(Component)]
struct UdpState;

fn build_hud(mut commands: Commands) {
    commands.spawn((Camera2d, DespawnOnExit(Connected(false))));

    commands.spawn((
        Name::new("HUD Root"),
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Start,
            align_content: AlignContent::End,
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(px(20), px(10)),
            ..Default::default()
        },
        DespawnOnExit(AppState::Game),
        children![(
            text("", 18.,),
            UdpState,
            TextLayout::justify(Justify::End),
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
    con: Option<Res<ConnectionInfo>>,
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
        if let Some(con) = con {
            text.0.push('\n');
            text.0.push_str(&con.debug_state());
        }
    }
}

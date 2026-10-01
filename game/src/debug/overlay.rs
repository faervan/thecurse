use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Game), build_overlay);

    app.add_systems(
        Update,
        (update_connection_info, update_player_info).run_if(in_state(AppState::Game)),
    );
}

#[derive(Component, Clone, Copy, Default)]
struct ConnectionDebugText;

#[derive(Component, Clone, Copy, Default)]
struct PlayerDebugText;

fn build_overlay(mut commands: Commands) {
    commands.spawn((Camera2d, DespawnOnExit(Connected(false))));

    commands.spawn_scene(bsn! {
        #OverlayRoot
        Node {
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Start,
            align_content: AlignContent::End,
            flex_direction: FlexDirection::Column,
            padding: UiRect::axes(px(20), px(10)),
        }
        template_value(DespawnOnExit(AppState::Game))
        Children [
            (
               Name::new("Connection info overlay")
               ConnectionDebugText
               text_block()
            ),
            (
               Name::new("Player info overlay")
               PlayerDebugText
               text_block()
            )
        ]
    });
}

fn text_block() -> impl Scene {
    bsn! {
        #Text
        TextLayout::justify(Justify::End)
        Node {
            right: Val::ZERO,
            align_self: AlignSelf::End
        }
        Text
        TextFont {
            font_size: FontSize::Px(18.)
        }
        TextColor(Color::WHITE)
    }
}

fn update_connection_info(
    udp: Res<Udp>,
    con: Option<Res<ConnectionInfo>>,
    mut text: Single<&mut Text, With<ConnectionDebugText>>,
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

fn update_player_info(
    mut text: Single<&mut Text, With<PlayerDebugText>>,
    player: Query<&Transform, With<MainCharacter>>,
    time: Res<Time>,
    mut timer: Local<Timer>,
    mut positions: Local<RingBuffer<Vec3, 8>>,
) {
    if timer.duration().is_zero() {
        *timer = Timer::new(Duration::from_millis(50), TimerMode::Repeating);
    }
    timer.tick(time.delta());
    if timer.just_finished() {
        let pos = player
            .single()
            .map(|pos| pos.translation)
            .unwrap_or_default();
        positions.push(pos);
        text.0 = format!(
            "x: {:.2} y: {:.2} z: {:.2}\nv: {:.2}",
            pos.x,
            pos.y,
            pos.z,
            (pos - positions.values().next().copied().unwrap_or_default()).length()
                / positions.len() as f32
                * 20.
        );
    }
}

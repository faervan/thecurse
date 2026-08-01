use bevy::ecs::system::IntoObserverSystem;

use crate::{menu::interaction::ButtonInteraction, prelude::*};

mod interaction;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(interaction::plugin);

    app.add_systems(OnEnter(AppState::Menu), build_main_menu);
    app.add_systems(
        Update,
        (|mut next_state: ResMut<NextState<AppState>>| next_state.set(AppState::Game)).run_if(
            in_state(AppState::Menu)
                .and(input_just_pressed(KeyCode::KeyG).or(input_just_pressed(KeyCode::KeyP))),
        ),
    );
    app.add_systems(
        Update,
        (|mut exit: MessageWriter<AppExit>| {
            exit.write(AppExit::Success);
        })
        .run_if(in_state(AppState::Menu).and(input_just_pressed(KeyCode::KeyQ))),
    );
}

fn build_main_menu(mut commands: Commands) {
    commands.spawn((Camera2d, DespawnOnExit(AppState::Menu)));

    commands.spawn((
        root_node(
            "Main Menu Root",
            JustifyContent::Center,
            AlignContent::Start,
            FlexDirection::Column,
            px(8),
            UiRect::horizontal(percent(10)),
        ),
        children![
            button(
                "Play",
                |_: On<Pointer<Click>>, mut next_state: ResMut<NextState<AppState>>| {
                    next_state.set(AppState::Game);
                }
            ),
            button(
                "Quit",
                |_: On<Pointer<Click>>, mut exit: MessageWriter<AppExit>| {
                    exit.write(AppExit::Success);
                }
            )
        ],
    ));
}

fn root_node(
    name: impl Into<std::borrow::Cow<'static, str>>,
    justify: JustifyContent,
    align: AlignContent,
    direction: FlexDirection,
    gap: Val,
    padding: UiRect,
) -> impl Bundle {
    container(
        name,
        percent(100),
        percent(100),
        justify,
        align,
        direction,
        gap,
        padding,
        Color::BLACK,
    )
}

fn container(
    name: impl Into<std::borrow::Cow<'static, str>>,
    width: Val,
    height: Val,
    justify: JustifyContent,
    align: AlignContent,
    direction: FlexDirection,
    gap: Val,
    padding: UiRect,
    background: Color,
) -> impl Bundle {
    (
        Name::new(name),
        Node {
            width,
            height,
            justify_content: justify,
            align_content: align,
            flex_direction: direction,
            padding,
            column_gap: gap,
            row_gap: gap,
            ..Default::default()
        },
        DespawnOnExit(AppState::Menu),
        BackgroundColor(background),
    )
}

fn button<A, E, B, M>(text: impl ToString, action: A) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    A: IntoObserverSystem<E, B, M>,
{
    let text = text.to_string();
    let action = IntoObserverSystem::into_system(action);
    let button_interaction = ButtonInteraction {
        normal: Color::srgb(0.5, 0.5, 0.5),
        hovered: Color::srgb(0.7, 0.7, 0.7),
        pressed: Color::srgb(1.0, 1.0, 1.0),
    };
    (
        Name::new("Button {text}"),
        Node::DEFAULT,
        DespawnOnExit(AppState::Menu),
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button {text} inner"),
                    Button,
                    Node {
                        padding: UiRect::axes(px(15), px(5)),
                        ..Default::default()
                    },
                    BackgroundColor(button_interaction.normal),
                    button_interaction,
                    children![(
                        Name::new("Button Text"),
                        Text(text),
                        TextFont::from_font_size(40.0),
                        TextColor(Color::BLACK),
                        Pickable::IGNORE,
                    )],
                ))
                .observe(action);
        })),
    )
}

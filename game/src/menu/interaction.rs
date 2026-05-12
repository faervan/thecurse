use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, button_interaction.run_if(in_state(AppState::Menu)));
}

#[derive(Component)]
pub struct ButtonInteraction {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}

fn button_interaction(
    query: Query<(&Interaction, &ButtonInteraction, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, button, mut color) in query {
        match interaction {
            Interaction::None => color.0 = button.normal,
            Interaction::Hovered => color.0 = button.hovered,
            Interaction::Pressed => color.0 = button.pressed,
        }
    }
}

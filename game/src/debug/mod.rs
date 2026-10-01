use bevy::dev_tools::render_debug::{RenderDebugOverlayEvent, handle_input, update_overlay};

use crate::prelude::*;

mod inspector;
mod overlay;
mod physics;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((inspector::plugin, physics::plugin, overlay::plugin));

    app.add_systems(
        Update,
        overwrite_debug_render_input
            .after(handle_input)
            .before(update_overlay),
    );
}

fn overwrite_debug_render_input(
    mut events: ResMut<Messages<RenderDebugOverlayEvent>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    events.clear();
    if input.just_pressed(KeyCode::F6) {
        events.write(RenderDebugOverlayEvent::CycleMode);
    }
    if input.just_pressed(KeyCode::F7) {
        events.write(RenderDebugOverlayEvent::CycleOpacity);
    }
}

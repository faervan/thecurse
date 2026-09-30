use bevy::{
    camera::{CameraOutputMode, visibility::RenderLayers},
    render::render_resource::BlendState,
};
use bevy_inspector_egui::{
    bevy_egui::{EguiGlobalSettings, EguiPlugin, PrimaryEguiContext},
    quick::WorldInspectorPlugin,
};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(EguiGlobalSettings {
        auto_create_primary_context: false,
        ..Default::default()
    });
    app.add_plugins(EguiPlugin::default());
    app.add_plugins(WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::F1)));

    app.add_systems(Startup, spawn_egui_camera);
}

fn spawn_egui_camera(mut commands: Commands) {
    // See
    // <https://github.com/vladbat00/bevy_egui/blob/fc3e83cfafa2b59bca9411c8cae5a1dea013d0cb/
    // examples/side_panel.rs#L166>
    commands.spawn((
        Name::new("Egui camera"),
        PrimaryEguiContext,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            output_mode: CameraOutputMode::Write {
                blend_state: Some(BlendState::ALPHA_BLENDING),
                clear_color: ClearColorConfig::None,
            },
            ..Default::default()
        },
        Camera2d,
        RenderLayers::none(),
    ));
}

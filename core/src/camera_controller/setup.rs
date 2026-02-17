use bevy::post_process::bloom::Bloom;

use crate::{
    camera_controller::{CameraController, CameraControllerAnchor, CameraControllerSettings},
    prelude::*,
};

pub fn spawn_camera(mut commands: Commands, settings: Res<CameraControllerSettings>) {
    let anchor = Vec3::Y;
    // Direction from anchor to the actual camera entity.
    let offset = Vec3::new(5., 6., 5.).normalize();

    commands
        .spawn((
            Name::new("Camera"),
            CameraControllerAnchor,
            ShapeCaster::default()
                .with_max_distance(0.)
                .with_max_hits(1)
                .with_query_filter(SpatialQueryFilter::from_mask(CollisionLayer::Environment)),
            Visibility::Visible,
            Transform::from_translation(anchor),
        ))
        .with_children(|parent| {
            parent.spawn((
                CameraController {
                    distance: settings.default_distance,
                    origin: Vec3::ZERO,
                },
                Camera3d::default(),
                Bloom::NATURAL,
                #[cfg(feature = "dev")]
                bevy_inspector_egui::bevy_egui::PrimaryEguiContext,
                Transform::from_translation(offset * settings.default_distance)
                    .looking_at(anchor, Vec3::Y),
            ));
        });
}

pub fn despawn(mut commands: Commands, query: Query<Entity, With<CameraControllerAnchor>>) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}

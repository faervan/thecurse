use std::ops::Range;

use bevy::{core_pipeline::prepass::DepthPrepass, post_process::bloom::Bloom};

use crate::prelude::*;

mod line_of_sight;
mod movement;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CameraControllerSettings>();

    app.add_systems(OnEnter(Connected(true)), spawn_camera);
    app.add_systems(
        Update,
        (
            movement::zoom,
            movement::rotate,
            movement::follow_player,
            line_of_sight::enforce
                .in_set(PhysicsSystems::Last)
                .after(movement::zoom),
        )
            .run_if(in_state(Connected(true))),
    );
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
/// The [`Entity`] with this component should always be a child of a [`CameraControllerAnchor`].
pub struct CameraController {
    /// The wanted distance from the camera origin. The actual distance might be smaller to prevent
    /// glitching into walls.
    distance: f32,
    /// Origin of the camera, meaning the point to which the [`CameraController`] is looking.
    /// Should be [`Vec3::ZERO`] (meaning it will look towards the [`CameraControllerAnchor`]),
    /// but may be offset if the [`CameraControllerAnchor`] is to close to an obstacle.
    origin: Vec3,
}

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub struct CameraControllerAnchor;

#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct CameraControllerSettings {
    /// Minimum and maximum distance the [`CameraController`] is allowed to have from the
    /// [`CameraControllerAnchor`]. The minimum may be ignored to enforce a line of sight to the
    /// [`CameraControllerAnchor`].
    pub distance: Range<f32>,
    pub default_distance: f32,
    pub zoom_speed: f32,
    /// [MouseWheel] provides scroll data in pixels for touchpads and in lines for mice. Pixel
    /// values get multiplied by both [`zoom_speed`] and [`touch_scroll_speed`], line values only by
    /// [zoom_speed].
    pub touch_scroll_speed: f32,
    /// How fast the camera zooms back out to its original distance after an obstacle in the line
    /// of sight got removed.
    pub zoom_recovery_speed: f32,
    /// The range of y values the normalized vector from the [`CameraControllerAnchor`] to the
    /// [`CameraController`] is allowed to have.
    pub y_range: Range<f32>,
    /// Horizontal and vertical rotation speed
    pub rotation_speed: Vec2,
}

impl Default for CameraControllerSettings {
    fn default() -> Self {
        Self {
            distance: 2_f32..10_f32,
            default_distance: 9.,
            zoom_speed: 1.2,
            touch_scroll_speed: 0.5,
            zoom_recovery_speed: 3.,
            y_range: 0.1..0.9,
            rotation_speed: Vec2::new(2., 0.5),
        }
    }
}

fn spawn_camera(mut commands: Commands, settings: Res<CameraControllerSettings>) {
    let anchor = Vec3::Y;
    // Direction from anchor to the actual camera entity.
    let offset = Vec3::new(5., 6., 5.).normalize();

    let camera_pos =
        Transform::from_translation(offset * settings.default_distance).looking_at(anchor, Vec3::Y);

    commands.spawn_scene(bsn! {
        #GameCamera
        CameraControllerAnchor
        template_value(ShapeCaster::default()
            .with_max_distance(0.)
            .with_max_hits(1)
            .with_query_filter(SpatialQueryFilter::from_mask(GameLayer::ENVIRONMENT)))
        template_value(DespawnOnExit(Connected(true)))
        Visibility::Visible
        Transform::from_translation(anchor)
        Children [
            CameraController
            Camera3d::default()
            PhysicsPickable
            IsDefaultUiCamera
            DepthPrepass
            Bloom::NATURAL
            template_value(camera_pos)
        ]
    });
}

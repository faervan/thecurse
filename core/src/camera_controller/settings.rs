use std::ops::Range;

use crate::prelude::*;

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
            y_range: -0.1..0.9,
            rotation_speed: Vec2::new(2., 0.5),
        }
    }
}

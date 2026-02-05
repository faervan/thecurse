use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::{
    camera_controller::{CameraController, CameraControllerSettings},
    prelude::*,
};

pub fn zoom(
    mut wheel_event: MessageReader<MouseWheel>,
    settings: Res<CameraControllerSettings>,
    mut camera: Single<(&mut Transform, &mut CameraController)>,
) {
    for scroll in wheel_event.read() {
        let y = match scroll.unit {
            MouseScrollUnit::Line => scroll.y,
            MouseScrollUnit::Pixel => scroll.y * settings.touch_scroll_speed,
        };
        let change = camera.0.translation.normalize() * -y * settings.zoom_speed;
        // The following is needed because else it is possible for pixel values to be big enough
        // for the camera to wrap to the other side
        // It is also needed to prevent scrolling when the player is so close to an obstacle that
        // the camera origin is offset.
        if camera.0.translation.x.abs() > change.x.abs() {
            camera.0.translation = (camera.0.translation + change)
                .clamp_length(settings.distance.start, settings.distance.end);
            camera.1.distance = camera.0.translation.length();
        }
    }
}

pub fn rotate(
    mut move_event: MessageReader<MouseMotion>,
    mouse_key: Res<ButtonInput<MouseButton>>,
    settings: Res<CameraControllerSettings>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut camera: Single<(&mut Transform, &mut CameraController)>,
) {
    if mouse_key.just_pressed(MouseButton::Right) {
        cursor.visible = false;
        cursor.grab_mode = CursorGrabMode::Locked;
    } else if mouse_key.just_released(MouseButton::Right) {
        cursor.visible = true;
        cursor.grab_mode = CursorGrabMode::None;
    }

    if mouse_key.pressed(MouseButton::Right) {
        for motion in move_event.read() {
            let yaw = -motion.delta.x * 0.002 * settings.rotation_speed.x;
            let pitch = -motion.delta.y * 0.002 * settings.rotation_speed.y;
            let offset_from_origin = camera.0.translation - camera.1.origin;
            let rotation = (
                // horizontal change
                Quat::from_axis_angle(Vec3::Y, yaw)
                // vertical change
                * Quat::from_axis_angle(
                    -(offset_from_origin)
                        .with_y(0.)
                        .any_orthogonal_vector(),
                    pitch
                )
            ) * camera.0.rotation;

            let origin = camera.1.origin;
            let mut direction = rotation * Vec3::Z;
            direction.y = direction
                .y
                .clamp(settings.y_range.start, settings.y_range.end);

            if camera.1.origin == Vec3::ZERO {
                camera.0.translation = direction.normalize() * camera.0.translation.length();
            } else {
                camera.1.origin = -direction.normalize();
            }
            camera.0.look_at(origin, Vec3::Y);
        }
    }
}

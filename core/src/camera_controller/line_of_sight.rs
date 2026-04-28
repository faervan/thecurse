use crate::{camera_controller::CameraControllerSettings, prelude::*};

/// TODO: toggle player visibility
pub fn enforce(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut anchor: Single<(&mut ShapeCaster, &ShapeHits), With<CameraControllerAnchor>>,
    mut camera: Single<(
        &mut Transform,
        &GlobalTransform,
        Ref<Projection>,
        &mut CameraController,
    )>,
    settings: Res<CameraControllerSettings>,
    // mut player: Query<&mut Visibility, With<MainCharacterM>>,
) {
    anchor.0.max_distance = camera.3.distance;
    if let Ok(direction) = Dir3::new(camera.0.translation - camera.3.origin) {
        anchor.0.direction = direction;
        anchor.0.shape_rotation = camera.0.rotation * Quat::from_axis_angle(Vec3::Y, PI / 2.);
    }

    if camera.2.is_changed()
        && let Projection::Perspective(projection) = &*camera.2
    {
        let height = 2. * projection.near * ops::tan(projection.fov * 0.5);
        let width = height * projection.aspect_ratio;
        anchor.0.shape = Collider::cuboid(width * 2., height * 2., 2.);
    }

    if let Some(hit) = anchor.1.first() {
        if hit.distance == 0. {
            if spatial_query
                .cast_shape(
                    &anchor.0.shape,
                    camera.1.translation(),
                    Quat::from_rotation_arc(Vec3::Z, hit.point1 - camera.1.translation())
                        * Quat::from_axis_angle(Vec3::Y, PI / 2.),
                    Dir3::new_unchecked((hit.point1 - camera.1.translation()).normalize()),
                    &ShapeCastConfig::default()
                        .with_max_distance((hit.point1 - camera.1.translation()).length()),
                    &SpatialQueryFilter::from_mask(CollisionLayer::Environment),
                )
                .is_none_or(|hit| hit.distance == 0.)
            {
                camera.0.translation = Vec3::ZERO;
                camera.3.origin = camera.0.rotation * Vec3::NEG_Z;
                // if let Ok(mut visibility) = player.single_mut()
                //     && *visibility != Visibility::Hidden
                // {
                //     *visibility = Visibility::Hidden;
                // }
            }
            return;
        }
        camera.0.translation = (camera.0.translation - camera.3.origin).normalize() * hit.distance;
        camera.3.origin = Vec3::ZERO;
        // if let Ok(mut visibility) = player.single_mut()
        //     && *visibility != Visibility::Visible
        // {
        //     *visibility = Visibility::Visible;
        // }
    } else if camera.0.translation.length() < camera.3.distance {
        let direction = (camera.0.translation - camera.3.origin).normalize();
        camera.0.translation = (camera.0.translation
            + direction * camera.3.distance * time.delta_secs() * settings.zoom_recovery_speed)
            .clamp_length_max(camera.3.distance);
        camera.3.origin = Vec3::ZERO;
        // if let Ok(mut visibility) = player.single_mut()
        //     && *visibility != Visibility::Visible
        // {
        //     *visibility = Visibility::Visible;
        // }
    }
}

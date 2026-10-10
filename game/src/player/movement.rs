use std::ops::Sub;

use dreamgame_core::player::PLAYER_MOVEMENT_SPEED;

use crate::{
    player::{InnerMainCharacter, animation::PlayerAnimationController},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (movement, apply_translation_correction)
            .chain()
            .run_if(in_state(Connected(true))),
    );
}

fn movement(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut con: ResMut<ConnectionInfo>,
    mut main_character: Single<(&mut LinearVelocity, &mut Transform, &mut MainCharacter)>,
    mut inner_main_character: Single<
        (&mut Transform, &mut PlayerAnimationController),
        (With<InnerMainCharacter>, Without<MainCharacter>),
    >,
    camera: Single<
        &Transform,
        (
            With<CameraController>,
            Without<MainCharacter>,
            Without<InnerMainCharacter>,
        ),
    >,
) {
    let (velocity, transform, main_character) = &mut *main_character;
    let (inner_transform, animation_controller) = &mut *inner_main_character;

    let offset = transform
        .translation
        .sub(Vec3::from_array(con.predicted_future_state.translation));
    if offset.length() > 0.1 {
        main_character.action_timer.tick(time.delta());
        if main_character.action_timer.just_finished() {
            con.predicted_future_state.translation = transform.translation.to_array();
            con.add_action(PlayerAction::Movement {
                offset: offset.to_array(),
            });
        }
    }

    let mut direction = Vec3::ZERO;
    if input.pressed(KeyCode::KeyW) {
        direction.z -= 1.;
    }
    if input.pressed(KeyCode::KeyS) {
        direction.z += 1.;
    }
    if input.pressed(KeyCode::KeyA) {
        direction.x -= 1.;
    }
    if input.pressed(KeyCode::KeyD) {
        direction.x += 1.;
    }

    if direction == Vec3::ZERO {
        if main_character.last_movement_direction != Vec3::ZERO {
            main_character.last_movement_direction = Vec3::ZERO;
            velocity.x = 0.;
            velocity.z = 0.;
        }
        animation_controller.set_to_idle();
        return;
    }
    animation_controller.set_to_running();

    let past = transform.rotation;
    let forward = Quat::from_rotation_arc(Vec3::NEG_Z, camera.translation.with_y(0.).normalize());
    transform.rotation = forward;
    transform.rotate_y((-direction.x).atan2(-direction.z));

    if direction != main_character.last_movement_direction {
        inner_transform.rotation = transform.rotation.inverse() * past;
        main_character.last_movement_direction = direction;
    }

    direction = (camera.rotation * direction).with_y(0.).normalize() * PLAYER_MOVEMENT_SPEED * 1.2;

    velocity.x = direction.x;
    velocity.z = direction.z;
}

fn apply_translation_correction(
    time: Res<Time>,
    main_character: Single<&Transform, (With<MainCharacter>, Without<InnerMainCharacter>)>,
    mut inner: Single<(&mut Transform, &mut InnerMainCharacter)>,
) {
    let (inner_transform, inner_character) = &mut *inner;
    let correction = &mut inner_character.translation_correction;
    let len = correction.length();
    if len > 0.05 {
        let diff = *correction * time.delta_secs() * 10.;
        *correction -= diff.clamp_length_max(len);
    } else {
        *correction = Vec3::ZERO;
    }
    inner_transform.translation = main_character.rotation.inverse() * *correction;
    let angle = inner_transform.rotation.angle_between(Quat::IDENTITY);
    if angle > 0.05 {
        inner_transform.rotation = inner_transform
            .rotation
            .rotate_towards(Quat::IDENTITY, angle * time.delta_secs() * 20.);
    } else {
        inner_transform.rotation = Quat::IDENTITY
    }
}

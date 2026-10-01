use dreamgame_core::{helpers::approx_eq, player::PLAYER_MOVEMENT_SPEED};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, movement.run_if(in_state(Connected(true))));
}

fn movement(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut con: ResMut<ConnectionInfo>,
    mut player: Query<(&mut LinearVelocity, &mut Transform, &mut MainCharacter)>,
    camera: Single<&Transform, (With<CameraController>, Without<MainCharacter>)>,
) {
    let Ok((mut velocity, player_pos, mut player)) = player.single_mut() else {
        return;
    };

    let translation = player_pos.translation.to_array();
    if !approx_eq(translation, con.predicted_future_state.translation) {
        player.action_timer.tick(time.delta());
        if player.action_timer.just_finished() {
            con.add_action(PlayerAction::Movement {
                destination: translation,
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
        if player.last_movement_direction != Vec3::ZERO {
            player.last_movement_direction = Vec3::ZERO;
            velocity.x = 0.;
            velocity.z = 0.;
        }
        return;
    }

    // let past = player_pos.rotation;
    // let forward = Quat::from_rotation_arc(Vec3::NEG_Z, camera.translation.with_y(0.).normalize());
    // player_pos.rotation = forward;
    // player_pos.rotate_y((-direction.x).atan2(-direction.z));

    if direction != player.last_movement_direction {
        // Rotate the inner armature entity that is the root of the character mesh back to the
        // rotation the player had previously, then catch up smoothly.
        // Directly animating the character rotation would logically make more sense, but I
        // dislike the input delay it brings (player presses left, but the character walks a
        // curve while smoothly rotating to the left).
        // if let Ok(mut armature_transform) = armatures.get_mut(**target) {
        //     armature_transform.rotation = Quat::from_rotation_arc(
        //         player_pos.rotation * Vec3::NEG_Z,
        //         past * armature_transform.rotation * Vec3::NEG_Z,
        //     );
        //
        //     commands.entity(**target).transition(Quat::IDENTITY, 100);
        // }
        player.last_movement_direction = direction;
    }

    direction = (camera.rotation * direction).with_y(0.).normalize() * PLAYER_MOVEMENT_SPEED;

    velocity.x = direction.x;
    velocity.z = direction.z;
}

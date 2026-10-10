use crate::{
    player::{STATE_TRANSITION_DURATION, animation::PlayerAnimationController},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        apply_scripted_state.run_if(in_state(Connected(true))),
    );
}

fn apply_scripted_state(
    time: Res<Time>,
    players: Query<(
        &mut ScriptedPlayer,
        &mut PlayerAnimationController,
        &mut Transform,
    )>,
) {
    for (mut player, mut animation_controller, mut transform) in players {
        let next_state = if let Some(next) = player.next_state {
            next
        } else {
            player.cache_timer.tick(time.delta());
            if player.cache_timer.is_finished()
                && let Some((next, tick_count)) = player.state_cache.pop_front()
            {
                let cache_len = player.state_cache.len() as u32;
                player
                    .transition_timer
                    .set_duration(STATE_TRANSITION_DURATION * tick_count / cache_len.max(1));
                player.cache_timer.reset();
                player.next_state = Some(next);
                next
            } else {
                animation_controller.set_to_idle();
                continue;
            }
        };
        player.transition_timer.tick(time.delta());

        let prev = Vec3::from_array(player.last_state.translation);
        let next = Vec3::from_array(next_state.translation);

        let diff = next - prev;
        let translation = prev + diff * player.transition_timer.fraction();
        transform.translation = translation;
        if let Some(dir) = diff.try_normalize()
            && dir.y < 0.95
        {
            let target_rotation = Quat::from_rotation_arc(Vec3::Z, dir);
            let angle = transform.rotation.angle_between(target_rotation);
            if angle > 0.05 {
                transform.rotation = transform
                    .rotation
                    .rotate_towards(target_rotation, angle * time.delta_secs() * 20.)
            } else {
                transform.rotation = target_rotation;
            }
        }

        const MIN_DISTANCE_SQUARED: f32 = 0.05 * 0.05;
        match diff.length_squared() > MIN_DISTANCE_SQUARED {
            true => animation_controller.set_to_running(),
            false => animation_controller.set_to_idle(),
        }

        if player.transition_timer.just_finished() {
            player.transition_timer.reset();
            player.last_state = next_state;
            match player.state_cache.pop_front() {
                Some((next, tick_count)) => {
                    let cache_len = player.state_cache.len() as u32;
                    player
                        .transition_timer
                        .set_duration(STATE_TRANSITION_DURATION * tick_count / cache_len.max(1));
                    player.next_state = Some(next);
                }
                None => {
                    animation_controller.set_to_idle();
                    player.next_state = None;
                }
            }
        }
    }
}

use crate::{player::STATE_TRANSITION_DURATION, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        apply_scripted_state.run_if(in_state(Connected(true))),
    );
}

fn apply_scripted_state(time: Res<Time>, players: Query<(&mut ScriptedPlayer, &mut Transform)>) {
    for (mut player, mut transform) in players {
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
                continue;
            }
        };
        player.transition_timer.tick(time.delta());

        let prev = Vec3::from_array(player.last_state.translation);
        let next = Vec3::from_array(next_state.translation);

        let diff = next - prev;
        let translation = prev + diff * player.transition_timer.fraction();
        transform.translation = translation;

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
                    player.next_state = None;
                }
            }
        }
    }
}

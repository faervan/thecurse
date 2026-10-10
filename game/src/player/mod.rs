use dreamgame_core::player::PLAYER_MOVEMENT_SEND_INTERVAL;

use crate::prelude::*;

mod character_asset;
pub mod cursor_target;
pub mod movement;
mod scripted;

const STATE_TRANSITION_DURATION: Duration =
    Duration::from_micros(SERVER_TIMESTEP.as_micros() as u64 * 4);

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        cursor_target::plugin,
        movement::plugin,
        scripted::plugin,
        character_asset::plugin,
    ));
}

#[derive(SceneComponent, FromTemplate, Reflect)]
#[reflect(Component)]
pub struct MainCharacter {
    pub id: ClientId,
    last_movement_direction: Vec3,
    action_timer: Timer,
}

#[derive(Component, Reflect, Debug, Default, Clone, Copy)]
#[reflect(Component)]
pub struct InnerMainCharacter;

impl MainCharacter {
    fn scene() -> impl Scene {
        bsn! {
            #MainCharacter
            MainCharacter {
                last_movement_direction: Vec3::ZERO,
                action_timer: Timer::new(PLAYER_MOVEMENT_SEND_INTERVAL, TimerMode::Repeating),
            }
            Visibility
            Player
            GameEntity
            Children [
                InnerMainCharacter
                @PlayerCharacter
            ]
        }
    }
}

#[derive(SceneComponent, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct ScriptedPlayer {
    pub id: ClientId,
    pub last_known_server_tick_id: u16,
    pub last_state: PlayerState,
    next_state: Option<PlayerState>,
    /// Stores the next state, as well as the count of server ticks since the last state
    state_cache: VecDeque<(PlayerState, u32)>,
    /// Timer for tracking the progress of a `last_state` -> `next_state` transition.
    transition_timer: Timer,
    /// Timer for building the cache when `next_state` is `None` and a new [`PlayerState`] has just
    /// been received.
    cache_timer: Timer,
}

impl ScriptedPlayer {
    pub fn push_state(&mut self, server_tick_id: u16, state: PlayerState) {
        let diff = server_tick_id.wrapping_sub(self.last_known_server_tick_id) as u32;
        self.last_known_server_tick_id = server_tick_id;
        self.state_cache.push_back((state, diff));
    }

    fn scene() -> impl Scene {
        bsn! {
            #ScriptedPlayer
            ScriptedPlayer {
                next_state: None,
                state_cache: VecDeque::new(),
                transition_timer: Timer::new(STATE_TRANSITION_DURATION, TimerMode::Once),
                cache_timer: Timer::new(Duration::from_millis(50), TimerMode::Once),
            }
            Children [
                @PlayerCharacter
            ]
        }
    }
}

#[derive(SceneComponent, Reflect, Default, Debug, Clone, Copy)]
#[reflect(Component)]
struct PlayerCharacter;

impl PlayerCharacter {
    fn scene() -> impl Scene {
        bsn! {
            #PlayerCharacter
            WorldAssetRoot("models/Player.glb#Scene0")
        }
    }
}

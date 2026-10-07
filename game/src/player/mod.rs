use dreamgame_core::player::PLAYER_MOVEMENT_SEND_INTERVAL;

use crate::prelude::*;

pub mod cursor_target;
pub mod movement;
mod scripted;

const STATE_TRANSITION_DURATION: Duration =
    Duration::from_micros(SERVER_TIMESTEP.as_micros() as u64 * 4);

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((cursor_target::plugin, movement::plugin, scripted::plugin));
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
#[require(PlayerCharacter)]
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
            ]
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(PlayerCharacter)]
pub struct ScriptedPlayer {
    pub id: ClientId,
    last_known_server_tick_id: u16,
    last_state: PlayerState,
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
    pub fn new(id: ClientId, state: PlayerState, server_tick_id: u16) -> Self {
        Self {
            id,
            last_known_server_tick_id: server_tick_id,
            last_state: state,
            next_state: None,
            state_cache: VecDeque::new(),
            transition_timer: Timer::new(STATE_TRANSITION_DURATION, TimerMode::Once),
            cache_timer: Timer::new(Duration::from_millis(50), TimerMode::Once),
        }
    }

    pub fn push_state(&mut self, server_tick_id: u16, state: PlayerState) {
        let diff = server_tick_id.wrapping_sub(self.last_known_server_tick_id) as u32;
        self.last_known_server_tick_id = server_tick_id;
        self.state_cache.push_back((state, diff));
    }
}

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
#[component(on_add)]
struct PlayerCharacter;

impl PlayerCharacter {
    fn on_add(mut world: DeferredWorld, hook: HookContext) {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh = meshes.add(Capsule3d::new(0.2, 1.7));

        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.1),
            ..Default::default()
        });

        world
            .commands()
            .entity(hook.entity)
            .insert((Mesh3d(mesh), MeshMaterial3d(material)));
    }
}

use crate::prelude::*;

pub const MOVEMENT_SPEED: f32 = 10.;
pub const AERIAL_MOVEMENT_FACTOR: f32 = 0.8;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(Transform)]
#[component(on_add)]
pub struct MovementState {
    /// The base movement direction like `(1, 0)`, `(-1, -1)`
    pub base_direction: Vec3,
    pub last_propagated_dir: Option<Vec2>,
    pub last_pos: Vec3,
    /// Whenever this [Timer] finishes or `direction` is vastly different from
    /// `last_propagated_dir`, the client will send a movement event to the server.
    pub propagation_timer: Timer,
}

impl Default for MovementState {
    fn default() -> Self {
        Self {
            base_direction: Vec3::ZERO,
            last_propagated_dir: None,
            last_pos: Vec3::ZERO,
            propagation_timer: Timer::new(Duration::from_millis(50), TimerMode::Repeating),
        }
    }
}

impl MovementState {
    fn on_add(mut world: DeferredWorld, hook: HookContext) {
        let pos = world.get::<Transform>(hook.entity).unwrap().translation;
        world.get_mut::<Self>(hook.entity).unwrap().last_pos = pos;
    }
}

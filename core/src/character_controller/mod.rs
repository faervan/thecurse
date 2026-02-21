use crate::prelude::*;

mod actions;
pub use actions::*;

mod action_hooks;
pub use action_hooks::*;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, character_movement.in_set(PhysicsSystems::First));
    }
}

pub type ActionIndex = usize;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct CharacterController {
    /// Registered [`CharacterAction`]`s of this [`CharacterController`]
    actions: Vec<CharacterAction>,
    /// Set of currently running [`CharacterAction`]`s, indices into the `action` field
    active_actions: HashSet<ActionIndex>,
    action_start_hooks: Vec<Vec<ActionHook>>,
    action_end_hooks: Vec<Vec<ActionHook>>,
}

impl CharacterController {
    pub fn from_actions(actions: Vec<CharacterAction>) -> (Self, Vec<ActionIndex>) {
        let len = actions.len();
        (
            Self {
                actions,
                active_actions: Default::default(),
                action_start_hooks: (0..len).map(|_| vec![]).collect(),
                action_end_hooks: (0..len).map(|_| vec![]).collect(),
            },
            (0..len).collect(),
        )
    }

    pub fn start_action(&mut self, action_id: ActionIndex) {
        self.active_actions.insert(action_id);
        for hook in &self.action_start_hooks[action_id] {
            hook.apply(&mut self.active_actions);
        }
    }

    pub fn end_action(&mut self, action_id: ActionIndex) {
        self.active_actions.remove(&action_id);
        for hook in &self.action_end_hooks[action_id] {
            hook.apply(&mut self.active_actions);
        }
    }

    pub fn add_action_start_hook(&mut self, action_id: ActionIndex, hook: ActionHook) {
        self.action_start_hooks[action_id].push(hook);
    }

    pub fn add_action_end_hook(&mut self, action_id: ActionIndex, hook: ActionHook) {
        self.action_end_hooks[action_id].push(hook);
    }
}

fn character_movement(
    time: Res<Time>,
    query: Query<(&mut CharacterController, &Transform, &mut LinearVelocity)>,
    other_transforms: Query<&Transform>,
) {
    for (mut controller, transform, mut velocity) in query {
        let mut v = Vec3::ZERO;
        for action_index in controller.active_actions.clone().into_iter() {
            let action = &mut controller.actions[action_index];
            if action.tick_action(transform, &other_transforms, &mut v, &time) {
                // This action finished
                controller.active_actions.remove(&action_index);
            }
        }
        velocity.0 = v;
    }
}

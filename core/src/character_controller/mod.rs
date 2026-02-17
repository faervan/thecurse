use crate::prelude::*;

mod actions;
pub use actions::*;

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
}

impl CharacterController {
    pub fn from_actions(actions: Vec<CharacterAction>) -> (Self, Vec<ActionIndex>) {
        let len = actions.len();
        (
            Self {
                actions,
                active_actions: Default::default(),
            },
            (0..len).collect(),
        )
    }

    pub fn start_action(&mut self, action_id: ActionIndex) {
        self.active_actions.insert(action_id);
    }

    pub fn end_action(&mut self, action_id: ActionIndex) {
        self.active_actions.remove(&action_id);
    }
}

fn character_movement(
    time: Res<Time>,
    query: Query<(&mut CharacterController, &Transform, &mut LinearVelocity)>,
) {
    for (mut controller, transform, mut velocity) in query {
        let mut v = Vec3::ZERO;
        for action_index in controller.active_actions.clone().into_iter() {
            let action = &mut controller.actions[action_index];
            if action.tick_action(&mut v, &time) {
                // This action finished
                controller.active_actions.remove(&action_index);
            }
        }
        velocity.0 = transform.rotation * v;
    }
}

use crate::{ActionIndex, prelude::*};

#[derive(Reflect, Debug)]
pub struct ActionHook {
    pub condition: ActionHookCondition,
    pub actions: Vec<ActionHookAction>,
}

impl ActionHook {
    pub fn apply(&self, active_actions: &mut HashSet<ActionIndex>) {
        if self.condition.check_condition(active_actions) {
            for action in &self.actions {
                action.apply(active_actions);
            }
        }
    }
}

#[derive(Reflect, Debug)]
pub enum ActionHookCondition {
    /// Always run the action when the hook is triggered
    Unconditional,
    IsRunning(ActionIndex),
    IsNotRunning(ActionIndex),
}

impl ActionHookCondition {
    fn check_condition(&self, active_actions: &HashSet<ActionIndex>) -> bool {
        match self {
            Self::Unconditional => true,
            Self::IsRunning(index) => active_actions.contains(index),
            Self::IsNotRunning(index) => !active_actions.contains(index),
        }
    }
}

#[derive(Reflect, Debug)]
/// Action to be performed if the [`ActionHookCondition`] is full filled
pub enum ActionHookAction {
    Insert(ActionIndex),
    Remove(ActionIndex),
}

impl ActionHookAction {
    fn apply(&self, active_actions: &mut HashSet<ActionIndex>) {
        match self {
            Self::Insert(action) => active_actions.insert(*action),
            Self::Remove(action) => active_actions.remove(action),
        };
    }
}

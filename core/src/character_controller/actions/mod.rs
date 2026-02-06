use std::time::Duration;

use crate::prelude::*;

mod action_layer;
pub use action_layer::ActionLayer;

pub struct CharacterAction {
    /// Description of spatial movement that is applied by this action.
    translation: TranslationGraph,
    /// The layer(s) this action belongs to. Also stores info about interruptibility and
    /// cancellability.
    layer: ActionLayer,
    /// The action layers that can be active simultaneously to this action.
    complements: ActionLayer,
    /// Animation to play
    animation: CharacterAnimation,
    /// Base duration of this action. The actual time may differ and some actions may be
    /// interrupted.
    duration: Duration,
    /// Trigger an entity event at the beginning of the action
    pre_event: Option<Box<dyn Fn(&mut EntityCommands)>>,
    /// Trigger an entity event at the end of the action
    post_event: Option<Box<dyn Fn(&mut EntityCommands)>>,
}

enum TranslationGraph {
    Constant(Vec3),
}

pub struct CharacterAnimation {
    pub index: AnimationNodeIndex,
    pub speed: f32,
}

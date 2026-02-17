use std::time::Duration;

use crate::prelude::*;

mod action_layer;
pub use action_layer::ActionLayer;

mod translation_graph;
use bevy::animation::RepeatAnimation;
pub use translation_graph::TranslationGraph;

#[derive(Default, Debug)]
pub struct CharacterAction {
    /// Description of spatial movement that is applied by this action.
    translation_graph: TranslationGraph,
    /// The layer(s) this action belongs to. Also stores info about interruptibility and
    /// cancellability.
    layer: ActionLayer,
    /// The action layers that can be active simultaneously to this action.
    complements: ActionLayer,
    /// Animation to play
    animation: CharacterAnimation,
    /// Timer of this action. Some actions may be interrupted.
    timer: Timer,
    /// Trigger an entity event at the beginning of the action
    pre_event: Option<CharacterActionEvent>,
    /// Trigger an entity event at the end of the action
    post_event: Option<CharacterActionEvent>,
}

impl CharacterAction {
    pub fn new(graph: TranslationGraph, animation: CharacterAnimation, duration: Duration) -> Self {
        Self {
            translation_graph: graph,
            animation,
            timer: Timer::new(duration, TimerMode::Once),
            ..Default::default()
        }
    }

    pub fn set_repeat(&mut self) {
        self.timer.set_mode(TimerMode::Repeating);
    }

    pub fn run_translation(&self, translation: &mut Vec3, time: &Time) {
        let delta = time.delta_secs() / self.timer.duration().as_secs_f32();
        let change = self.translation_graph.run_step(delta);
        *translation += change;
    }

    pub fn begin_animation(
        &self,
        transitions: &mut AnimationTransitions,
        player: &mut AnimationPlayer,
    ) {
        transitions
            .play(
                player,
                self.animation.index,
                self.animation.transition_duration,
            )
            .set_speed(self.animation.speed)
            .set_repeat(match self.timer.mode() {
                TimerMode::Once => RepeatAnimation::Never,
                TimerMode::Repeating => RepeatAnimation::Forever,
            });
    }
}

#[derive(Debug)]
pub enum CharacterActionEvent {
    ActionStarted,
    ActionFinished,
}

#[derive(Default, Debug)]
pub struct CharacterAnimation {
    pub index: AnimationNodeIndex,
    pub speed: f32,
    pub transition_duration: Duration,
}

impl CharacterAnimation {
    pub fn new(animation: AnimationNodeIndex) -> Self {
        Self {
            index: animation,
            speed: 1.,
            transition_duration: Duration::from_millis(100),
        }
    }
}

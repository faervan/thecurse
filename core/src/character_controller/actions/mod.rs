use crate::prelude::*;

mod action_layer;
pub use action_layer::ActionLayer;

mod translation_graph;
// use bevy::animation::RepeatAnimation;
pub use translation_graph::TranslationGraph;

#[derive(Reflect, Default, Debug)]
#[reflect(Default)]
pub struct CharacterAction {
    /// Description of spatial movement that is applied by this action.
    pub translation_graph: TranslationGraph,
    /// Multiplier for the [`TranslationGraph`]. If the action is moving, this would be speed.
    pub translation_multiplier: f32,
    #[reflect(ignore)]
    /// The layer(s) this action belongs to. Also stores info about interruptibility and
    /// cancellability.
    pub layer: ActionLayer,
    #[reflect(ignore)]
    /// The action layers that can be active simultaneously to this action.
    pub complements: ActionLayer,
    /// Animation to play
    pub animation: CharacterAnimation,
    /// Timer of this action. Some actions may be interrupted.
    pub timer: Timer,
    /// Trigger an entity event at the beginning of the action
    pub pre_event: Option<CharacterActionEvent>,
    /// Trigger an entity event at the end of the action
    pub post_event: Option<CharacterActionEvent>,
}

impl CharacterAction {
    pub fn new(graph: TranslationGraph, animation: CharacterAnimation, duration: Duration) -> Self {
        Self {
            translation_graph: graph,
            translation_multiplier: 1.,
            animation,
            timer: Timer::new(duration, TimerMode::Once),
            ..Default::default()
        }
    }

    pub fn set_translation_multiplier(mut self, multiplier: f32) -> Self {
        self.translation_multiplier = multiplier;
        self
    }

    pub fn set_repeat(mut self) -> Self {
        self.timer.set_mode(TimerMode::Repeating);
        self
    }

    pub fn start_action(
        &mut self,
        // transitions: &mut AnimationTransitions,
        // player: &mut AnimationPlayer,
    ) {
        self.timer.reset();
        // transitions
        //     .play(
        //         player,
        //         self.animation.index,
        //         self.animation.transition_duration,
        //     )
        //     .set_speed(self.animation.speed)
        //     .set_repeat(match self.timer.mode() {
        //         TimerMode::Once => RepeatAnimation::Never,
        //         TimerMode::Repeating => RepeatAnimation::Forever,
        //     });
    }

    /// Run the [`CharacterAction`] for the current frame, returning whether it finished
    pub fn tick_action(&mut self, translation: &mut Vec3, time: &Time) -> bool {
        let delta = time.delta_secs() / self.timer.duration().as_secs_f32();
        let change = self.translation_graph.run_step(delta);
        *translation += change * self.translation_multiplier;
        self.timer.tick(time.delta());
        self.timer.is_finished() && self.timer.mode() == TimerMode::Once
    }
}

#[derive(Reflect, Debug)]
pub enum CharacterActionEvent {
    ActionStarted,
    ActionFinished,
}

#[derive(Reflect, Debug, Default)]
#[reflect(Default)]
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

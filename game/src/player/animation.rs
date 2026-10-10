use dreamgame_core::player::PLAYER_MOVEMENT_SPEED;

use crate::{player::character_asset::PlayerCharacterHandle, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        update_player_animations.run_if(in_state(Connected(true))),
    );
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct PlayerAnimationController {
    animation_player: Entity,
    current: PlayerAnimation,
    next: Option<PlayerAnimation>,
}

#[derive(Reflect, Default, Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerAnimation {
    #[default]
    Idle,
    Running,
}

impl ChildEntityPointer for PlayerAnimationController {
    type Target = AnimationPlayer;
    fn new(entity: Entity) -> Self {
        Self::new(entity)
    }
}

impl PlayerAnimationController {
    pub fn new(animation_player: Entity) -> Self {
        Self {
            animation_player,
            current: PlayerAnimation::default(),
            next: None,
        }
    }

    pub fn set_to_idle(&mut self) {
        self.next = Some(PlayerAnimation::Idle);
    }

    pub fn set_to_running(&mut self) {
        self.next = Some(PlayerAnimation::Running);
    }
}

const MOVEMENT_ANIMATION_SPEED: f32 = 1. + PLAYER_MOVEMENT_SPEED * 0.1;
fn update_player_animations(
    character: Res<PlayerCharacterHandle>,
    controllers: Query<&mut PlayerAnimationController>,
    mut animation_players: Query<(&mut AnimationTransitions, &mut AnimationPlayer)>,
) {
    for mut controller in controllers {
        let next = controller.next.take();
        if let Some(next) = next
            && next != controller.current
        {
            controller.current = next;
            if let Ok((mut transitions, mut player)) =
                animation_players.get_mut(controller.animation_player)
            {
                match next {
                    PlayerAnimation::Idle => {
                        transitions.play(&mut player, character.idle, Duration::from_millis(100));
                    }
                    PlayerAnimation::Running => {
                        transitions
                            .play(&mut player, character.running, Duration::from_millis(100))
                            .set_speed(MOVEMENT_ANIMATION_SPEED)
                            .repeat();
                    }
                }
            }
        }
    }
}

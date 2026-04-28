use crate::prelude::*;

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(
            Update,
            (
                update_aerial_state,
                aerial_changed.after(update_aerial_state),
            )
                .run_if(in_state(game_state)),
        );
    }
}

#[derive(Component, Reflect, Debug, Default, PartialEq)]
#[reflect(Component)]
pub enum AerialState {
    #[default]
    Grounded,
    Jumping,
    Falling,
}

fn update_aerial_state(
    input: Res<ButtonInput<KeyCode>>,
    query: Query<(&mut AerialState, &mut LinearVelocity)>,
) {
    for (mut aerial, mut velocity) in query {
        let jump_pressed = input.just_pressed(KeyCode::Space);

        if velocity.y.abs() < 1. {
            if !jump_pressed {
                match *aerial {
                    AerialState::Grounded | AerialState::Jumping => {}
                    AerialState::Falling => *aerial = AerialState::Grounded,
                }
            } else if jump_pressed && *aerial != AerialState::Jumping {
                *aerial = AerialState::Jumping;
                velocity.y = 40.;
            }
            return;
        }

        if velocity.y < -1. && *aerial != AerialState::Falling {
            *aerial = AerialState::Falling;
            return;
        }
    }
}

fn aerial_changed(
    changed: Query<(&AerialState, &GltfAnimationTarget), Changed<AerialState>>,
    mut players: Query<(&mut AnimationTransitions, &mut AnimationPlayer)>,
    character: Res<PlayerCharacterHandle>,
) {
    for (aerial, target) in changed {
        if let Ok((mut transitions, mut player)) = players.get_mut(**target) {
            let animation = match aerial {
                AerialState::Grounded => character.idle,
                AerialState::Jumping => character.jumping,
                AerialState::Falling => character.falling,
            };
            transitions.play(&mut player, animation, Duration::from_millis(100));
        }
    }
}

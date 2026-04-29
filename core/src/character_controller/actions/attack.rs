use crate::{character_controller::actions::aerial::AerialState, prelude::*};

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(
            Update,
            (
                update_attack_state,
                attack_changes.after(update_attack_state),
            )
                .run_if(in_state(game_state)),
        );
    }
}

#[derive(Component, Reflect, Debug, Default, PartialEq)]
#[reflect(Component)]
pub enum AttackState {
    #[default]
    None,
    Attacking(Timer),
}

fn update_attack_state(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    query: Query<(&mut AttackState, &AerialState), With<MainCharacter>>,
) {
    for (mut attack_state, aerial) in query {
        if *attack_state == AttackState::None {
            if input.pressed(KeyCode::KeyQ) && *aerial == AerialState::Grounded {
                *attack_state =
                    AttackState::Attacking(Timer::new(Duration::from_millis(500), TimerMode::Once));
            }
        } else if let AttackState::Attacking(timer) = &mut *attack_state {
            timer.tick(time.delta());
            if timer.just_finished() {
                *attack_state = AttackState::None;
            }
        }
    }
}

fn attack_changes(
    changed: Query<(&AttackState, &GltfAnimationTarget), Changed<AttackState>>,
    mut players: Query<(&mut AnimationTransitions, &mut AnimationPlayer)>,
    character: Res<PlayerCharacterHandle>,
) {
    for (attack, target) in changed {
        if let Ok((mut transitions, mut player)) = players.get_mut(**target) {
            let animation = match attack {
                AttackState::None => character.idle,
                AttackState::Attacking(timer) if timer.elapsed().is_zero() => character.attack,
                _ => continue,
            };
            transitions
                .play(&mut player, animation, Duration::from_millis(100))
                .set_speed(2.);
        }
    }
}

use crate::{
    character_controller::{actions::aerial::AerialState, weapon::WeaponColliderHandle},
    prelude::*,
};

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(
            Update,
            ((update_attack_state, handle_interrupts), attack_changes)
                .chain()
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
    mut commands: Commands,
    changed: Query<
        (&AttackState, &GltfAnimationTarget, &WeaponColliderHandle),
        Changed<AttackState>,
    >,
    mut players: Query<(&mut AnimationTransitions, &mut AnimationPlayer)>,
    character: Res<PlayerCharacterHandle>,
) {
    for (attack, target, weapon_entity) in changed {
        let animation = match attack {
            AttackState::None => {
                commands
                    .entity(**weapon_entity)
                    .remove::<CollisionEventsEnabled>();
                character.idle
            }
            AttackState::Attacking(timer) if timer.elapsed().is_zero() => {
                commands
                    .entity(**weapon_entity)
                    .insert(CollisionEventsEnabled);
                character.attack
            }
            _ => continue,
        };
        if let Ok((mut transitions, mut player)) = players.get_mut(**target) {
            transitions
                .play(&mut player, animation, Duration::from_millis(100))
                .set_speed(2.);
        }
    }
}

fn handle_interrupts(
    mut interrupts: MessageReader<InterruptAction>,
    mut query: Query<&mut AttackState, With<MainCharacter>>,
) {
    for interrupt in interrupts.read() {
        for mut aerial in &mut query {
            match interrupt {
                InterruptAction::PlayerJumped => *aerial = AttackState::None,
            }
        }
    }
}

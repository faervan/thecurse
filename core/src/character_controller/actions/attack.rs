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
    Attacking {
        timer: Timer,
        ty: AttackType,
    },
}

#[derive(Reflect, Debug, PartialEq)]
pub enum AttackType {
    Normal,
    SwingBottom,
}

fn update_attack_state(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    query: Query<(&mut AttackState, &AerialState), With<MainCharacter>>,
) {
    for (mut attack_state, aerial) in query {
        if *attack_state == AttackState::None {
            if *aerial == AerialState::Grounded {
                if input.pressed(KeyCode::KeyQ) {
                    *attack_state = AttackState::Attacking {
                        timer: Timer::new(Duration::from_millis(500), TimerMode::Once),
                        ty: AttackType::Normal,
                    };
                } else if input.pressed(KeyCode::KeyE) {
                    *attack_state = AttackState::Attacking {
                        timer: Timer::new(Duration::from_millis(500), TimerMode::Once),
                        ty: AttackType::SwingBottom,
                    };
                }
            }
        } else if let AttackState::Attacking { timer, .. } = &mut *attack_state {
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
                commands.entity(**weapon_entity).remove::<Collider>();
                character.idle
            }
            AttackState::Attacking { timer, ty } if timer.elapsed().is_zero() => {
                commands
                    .entity(**weapon_entity)
                    .insert(Collider::cuboid(0.5, 5., 0.3));
                match ty {
                    AttackType::Normal => character.attack,
                    AttackType::SwingBottom => character.attack_bottom,
                }
            }
            _ => continue,
        };
        if let Ok((mut transitions, mut player)) = players.get_mut(**target) {
            transitions
                .play(&mut player, animation, Duration::from_millis(200))
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

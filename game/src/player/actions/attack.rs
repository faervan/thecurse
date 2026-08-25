use thecurse_core::creatures::player::{
    AerialState, AttackState, AttackType, attack_state_changes,
};

use crate::{
    player::{PlayerCharacterHandle, actions::InterruptAction},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            (update_attack_state, handle_interrupts),
            (attack_animation_changes, attack_state_changes),
        )
            .chain()
            .run_if(in_state(AppState::Game)),
    );
}

fn update_attack_state(
    input: Res<ButtonInput<KeyCode>>,
    query: Query<(
        Entity,
        &mut MainCharacter,
        &mut AttackState,
        &AerialState,
        &Transform,
    )>,
    mut cast_spell: MessageWriter<SpawnSpellVoid>,
    cursor_target: Res<CursorTargetPosition>,
) {
    for (entity, mut character, mut attack_state, aerial, pos) in query {
        if *attack_state == AttackState::None && *aerial == AerialState::Grounded {
            let mut attack = None;
            if input.pressed(KeyCode::KeyQ) {
                attack = Some(AttackType::Normal);
            } else if input.pressed(KeyCode::KeyE) {
                attack = Some(AttackType::SwingBottom);
            } else if input.pressed(KeyCode::KeyR)
                && let Some(position) = **cursor_target
            {
                attack = Some(AttackType::CastVoid);
                cast_spell.write(SpawnSpellVoid {
                    position,
                    caster: entity,
                });
            }

            if let Some(ty) = attack {
                // character.add_action(PlayerAction::Attack {
                //     ty,
                //     translation: pos.translation.to_array(),
                //     rotation: pos.rotation.to_array(),
                // });
                *attack_state = AttackState::Attacking {
                    timer: Timer::new(ty.duration(), TimerMode::Once),
                    ty,
                };
            }
        }
    }
}

fn attack_animation_changes(
    changed: Query<(&AttackState, &GltfAnimationTarget), Changed<AttackState>>,
    mut players: Query<(&mut AnimationTransitions, &mut AnimationPlayer)>,
    character: Res<PlayerCharacterHandle>,
) {
    for (attack, target) in changed {
        let animation = match attack {
            AttackState::None => character.idle,
            AttackState::Attacking { timer, ty } if timer.elapsed().is_zero() => match ty {
                AttackType::Normal => character.attack,
                AttackType::SwingBottom => character.attack_bottom,
                AttackType::CastVoid => character.idle,
            },
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
        for mut attack in &mut query {
            if *attack == AttackState::None {
                continue;
            }
            match interrupt {
                InterruptAction::PlayerJumped => *attack = AttackState::None,
            }
        }
    }
}

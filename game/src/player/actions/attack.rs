use crate::{
    player::{
        PlayerCharacterHandle,
        actions::{InterruptAction, aerial::AerialState},
    },
    prelude::*,
    weapon::BASIC_SWORD_DAMAGE,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        ((update_attack_state, handle_interrupts), attack_changes)
            .chain()
            .run_if(in_state(AppState::Game)),
    );
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
    CastVoid,
}

fn update_attack_state(
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &mut AttackState, &AerialState), With<MainCharacter>>,
    mut cast_spell: MessageWriter<SpawnSpellVoid>,
    cursor_target: Res<CursorTargetPosition>,
) {
    for (entity, mut attack_state, aerial) in query {
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
                } else if input.pressed(KeyCode::KeyR)
                    && let Some(position) = **cursor_target
                {
                    *attack_state = AttackState::Attacking {
                        timer: Timer::new(Duration::from_millis(200), TimerMode::Once),
                        ty: AttackType::CastVoid,
                    };
                    cast_spell.write(SpawnSpellVoid {
                        position,
                        caster: entity,
                    });
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
        (
            Entity,
            &AttackState,
            &GltfAnimationTarget,
            &WeaponColliderHandle,
        ),
        Changed<AttackState>,
    >,
    mut players: Query<(&mut AnimationTransitions, &mut AnimationPlayer)>,
    character: Res<PlayerCharacterHandle>,
) {
    for (entity, attack, target, weapon_entity) in changed {
        let animation = match attack {
            AttackState::None => {
                commands
                    .entity(**weapon_entity)
                    .remove::<(Collider, Sensor)>();
                character.idle
            }
            AttackState::Attacking { timer, ty } if timer.elapsed().is_zero() => {
                commands.entity(**weapon_entity).insert((
                    Collider::cuboid(0.5, 5., 0.3),
                    Sensor,
                    DamageSource::new(entity, BASIC_SWORD_DAMAGE),
                ));
                match ty {
                    AttackType::Normal => character.attack,
                    AttackType::SwingBottom => character.attack_bottom,
                    AttackType::CastVoid => character.idle,
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

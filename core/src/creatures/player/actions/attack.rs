use crate::{creatures::BASIC_SWORD_DAMAGE, prelude::*};

#[derive(Component, Reflect, Deref)]
pub struct PlayerMeleeAttackCollider(pub Entity);

#[derive(ByteRepr, Reflect, Debug, PartialEq, Clone, Copy)]
pub enum AttackType {
    Normal,
    SwingBottom,
    CastVoid,
}

impl AttackType {
    pub fn duration(&self) -> Duration {
        Duration::from_millis(match self {
            Self::Normal | Self::SwingBottom => 500,
            Self::CastVoid => 200,
        })
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

pub fn attack_state_changes(
    mut commands: Commands,
    time: Res<Time>,
    query: Query<(Entity, &mut AttackState, &PlayerMeleeAttackCollider)>,
) {
    for (entity, mut state, collider) in query {
        match &mut *state {
            AttackState::None => continue,
            AttackState::Attacking { timer, ty } => {
                if timer.elapsed().is_zero() {
                    match ty {
                        AttackType::CastVoid => {}
                        AttackType::Normal | AttackType::SwingBottom => {
                            commands.entity(**collider).insert((
                                Transform::from_xyz(0., 0., 1.),
                                Collider::cuboid(1.2, 1.5, 1.4),
                                Sensor,
                                DamageSource::new(entity, BASIC_SWORD_DAMAGE),
                            ));
                        }
                    }
                }
                timer.tick(time.delta());
                if timer.is_finished() {
                    *state = AttackState::None;
                    commands
                        .entity(**collider)
                        .remove::<(Collider, Sensor, Position, Rotation)>();
                }
            }
        }
    }
}

use crate::prelude::*;

mod attack;
pub use attack::*;

mod movement;
pub use movement::*;

mod aerial;
pub use aerial::*;

#[derive(Bundle, Default)]
pub struct CharacterActions {
    pub aerial: AerialState,
    pub movement: MovementState,
    pub attack: AttackState,
}

#[derive(ByteRepr, Debug, Clone)]
pub enum PlayerAction {
    Attack {
        ty: AttackType,
        translation: [f32; 3],
        rotation: [f32; 4],
    },
    Movement,
}

impl PlayerAction {
    pub fn apply(self, entity: Entity, commands: &mut Commands) {
        match self {
            Self::Attack {
                ty,
                translation,
                rotation,
            } => {
                commands
                    .entity(entity)
                    .insert(AttackState::Attacking {
                        timer: Timer::new(ty.duration(), TimerMode::Once),
                        ty,
                    })
                    .entry::<Transform>()
                    .and_modify(move |mut pos| {
                        pos.translation = Vec3::from_array(translation);
                        pos.rotation = Quat::from_array(rotation)
                    });
            }
            Self::Movement => todo!(),
        }
    }
}

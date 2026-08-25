use crate::prelude::*;

mod attack;
pub use attack::*;

mod movement;
pub use movement::*;

mod aerial;
pub use aerial::*;

#[derive(ByteRepr, Debug, Clone, Reflect)]
pub enum PlayerAction {
    Attack {
        ty: AttackType,
        translation: [f32; 3],
        rotation: [f32; 4],
    },
    Movement {
        direction: [f32; 2],
        duration_millis: u8,
    },
}

#[derive(ByteRepr, Debug, Clone)]
pub enum PlayerActionBroadcast {
    Attack {
        ty: AttackType,
        translation: [f32; 3],
        rotation: [f32; 4],
    },
    Movement {
        destination: [f32; 3],
        just_started: bool,
    },
}

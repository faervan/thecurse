use crate::prelude::*;

mod attack;
pub use attack::*;

mod movement;
pub use movement::*;

mod aerial;
pub use aerial::*;

#[derive(ByteRepr, Debug, Clone)]
pub enum PlayerAction {
    Attack {
        ty: AttackType,
        translation: [f32; 3],
        rotation: [f32; 4],
    },
    Movement {
        origin: [f32; 2],
        direction: [f32; 2],
        destination: [f32; 2],
        duration_secs: f32,
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
        duration_secs: f32,
    },
}

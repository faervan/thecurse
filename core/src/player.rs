use crate::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(
    RigidBody::Dynamic,
    Collider::cuboid(0.5, 1.94, 0.2),
    CollisionLayers::new(
        GameLayer::CREATURE,
        GameLayer::DEFAULT | GameLayer::ENVIRONMENT | GameLayer::DAMAGE_SOURCE,
    ),
    GravityScale(10.),
    LockedAxes::ROTATION_LOCKED
)]
pub struct Player;

pub const PLAYER_MOVEMENT_SPEED: f32 = 10.;

#[derive(ByteRepr, Debug, PartialEq, Clone, Copy)]
pub struct PlayerState {
    pub translation: [f32; 3],
}

#[derive(ByteRepr, Debug, Clone, Copy)]
pub enum PlayerAction {
    Movement { destination: [f32; 3] },
}

impl PlayerState {
    pub fn apply(&mut self, action: PlayerAction) {
        match action {
            PlayerAction::Movement { destination } => self.translation = destination,
        }
    }
}

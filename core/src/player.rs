use std::ops::Add;

use crate::prelude::*;

#[derive(Component, Reflect, Default, Clone, Copy)]
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
pub const PLAYER_MOVEMENT_SEND_INTERVAL: Duration = Duration::from_millis(50);

#[derive(ByteRepr, Reflect, Debug, PartialEq, Clone, Copy)]
pub struct PlayerState {
    pub translation: [f32; 3],
}

#[derive(ByteRepr, Reflect, Debug, Clone, Copy)]
pub enum PlayerAction {
    Movement { offset: [f32; 3] },
}

impl PlayerState {
    pub fn apply(&mut self, action: PlayerAction) {
        match action {
            PlayerAction::Movement { offset } => {
                self.translation = Vec3::from_array(self.translation)
                    .add(Vec3::from_array(offset))
                    .to_array()
            }
        }
    }
}

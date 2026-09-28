use crate::prelude::*;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct PlayerActionQueue {
    actions: VecDeque<PlayerAction>,
}

#[derive(Reflect, Debug)]
pub enum PlayerAction {}

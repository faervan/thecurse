use crate::prelude::*;

pub fn plugin(app: &mut App) {}

#[derive(Component, Debug, Clone)]
pub struct Client {
    pub id: ClientId,
    pub addr: SocketAddr,
}

#[derive(Component, Debug, Clone)]
pub struct ClientCharacter {}

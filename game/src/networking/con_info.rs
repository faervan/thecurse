use crate::prelude::*;

#[derive(Resource)]
pub struct ConnectionInfo {
    last_processed_action: u16,
    client_id: ClientId,
    entity: Entity,
    pub clients: HashMap<ClientId, Entity>,
}

impl ConnectionInfo {
    pub fn new(id: ClientId, entity: Entity) -> Self {
        Self {
            last_processed_action: u16::MAX,
            client_id: id,
            entity,
            clients: HashMap::new(),
        }
    }
}

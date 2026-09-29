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

    pub fn debug_state(&self) -> String {
        let main_character = format!("You (#{}), {}", self.client_id.0, self.entity);
        let clients = self
            .clients
            .iter()
            .map(|(id, entity)| format!("Client #{}, {entity}", id.0))
            .collect::<Vec<String>>()
            .join("\n");
        [main_character, clients].join("\n")
    }
}

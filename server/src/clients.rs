use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_resource::<ConnectedClients>();
    app.add_systems(Update, receive_client_changes);
}

#[derive(Resource)]
pub struct TcpClientChanges {
    pub rx: Receiver<TcpClientChange>,
}

pub enum TcpClientChange {
    ClientConnect { id: ClientId },
    ClientDisconnect { id: ClientId },
}

#[derive(Resource, Default)]
pub struct ConnectedClients {
    pub client_entities: HashMap<ClientId, Entity>,
    pub addr_clients: HashMap<SocketAddr, ClientId>,
    pub client_addrs: HashMap<ClientId, SocketAddr>,
}

#[derive(Component, Deref, Debug)]
pub struct ClientAddr(pub SocketAddr);

fn receive_client_changes(
    mut commands: Commands,
    changes: Res<TcpClientChanges>,
    mut clients: ResMut<ConnectedClients>,
) {
    while let Ok(change) = changes.rx.try_recv() {
        match change {
            TcpClientChange::ClientConnect { id } => {
                let entity = commands
                    .spawn((Player, Name::new(format!("Player #{}", id.0)), id))
                    .id();
                clients.client_entities.insert(id, entity);
            }
            TcpClientChange::ClientDisconnect { id } => {
                clients.client_entities.remove(&id);
                if let Some(addr) = clients.client_addrs.remove(&id) {
                    clients.addr_clients.remove(&addr);
                }
            }
        }
    }
}

use crate::prelude::*;

pub fn plugin(_app: &mut App) {}

#[derive(Default)]
pub struct ConnectedClients {
    addr_clients: HashMap<SocketAddr, ConnectedClient>,
    client_addrs: HashMap<ClientId, SocketAddr>,
    pending_action_broadcasts: Vec<(ClientId, PlayerActionBroadcast)>,
}

pub struct ConnectedClient {
    pub id: ClientId,
    pub entity: Entity,
    pub last_processed_action: u16,
    pub pending_messages: VecDeque<UdpMsgToClient>,
}

impl ConnectedClients {
    pub fn insert(
        &mut self,
        id: ClientId,
        last_processed_action: u16,
        addr: SocketAddr,
        entity: Entity,
        translation: [f32; 3],
    ) {
        self.client_addrs.insert(id, addr);
        self.addr_clients.insert(
            addr,
            ConnectedClient {
                id,
                entity,
                last_processed_action,
                pending_messages: VecDeque::new(),
            },
        );
        self.addr_clients
            .iter_mut()
            .filter(|(a, _)| **a != addr)
            .for_each(|(_, client)| {
                client
                    .pending_messages
                    .push_back(UdpMsgToClient::PlayerConnected { id, translation })
            });
    }

    pub fn broadcast_action(&mut self, client_id: ClientId, action: PlayerActionBroadcast) {
        self.pending_action_broadcasts.push((client_id, action));
    }

    pub fn get_mut(&mut self, addr: &SocketAddr) -> Option<&mut ConnectedClient> {
        debug_assert!(self.addr_clients.contains_key(addr));
        self.addr_clients.get_mut(addr)
    }

    pub fn flush_pending_messages(
        &mut self,
        udp: &mut MultiUdpCommunicator<UdpMsgToClient, UdpMsgToServer, PROTOCOL_VERSION>,
        server_broadcast_tick_id: u16,
    ) {
        for mut com in udp.iter_mut() {
            let Some(ConnectedClient {
                last_processed_action,
                pending_messages,
                id,
                ..
            }) = self.addr_clients.get_mut(&com.addr)
            else {
                continue;
            };
            for msg in pending_messages.drain(..) {
                com.write_ordered(msg);
            }
            for (client_id, action) in self.pending_action_broadcasts.clone() {
                if *id == client_id {
                    continue;
                }
                com.write_ordered(UdpMsgToClient::PlayerAction {
                    client_id,
                    last_processed_action: *last_processed_action,
                    server_broadcast_tick_id,
                    action,
                });
            }
        }
        self.pending_action_broadcasts.clear();
    }

    pub fn remove(&mut self, addr: SocketAddr) -> Option<Entity> {
        let client = self.addr_clients.remove(&addr)?;
        self.client_addrs.remove(&client.id).unwrap();

        info!("Removed client {:?}", client.id);

        // Broadcast disconnect
        for c in self.addr_clients.values_mut() {
            c.pending_messages
                .push_back(UdpMsgToClient::PlayerDisconnected { id: client.id })
        }

        Some(client.entity)
    }
}

#[derive(Component, Deref, Debug)]
pub struct ClientAddr(pub SocketAddr);

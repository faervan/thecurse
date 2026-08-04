use std::collections::VecDeque;

use crate::prelude::*;

pub fn plugin(_app: &mut App) {}

#[derive(Default)]
pub struct ConnectedClients {
    client_entities: HashMap<ClientId, Entity>,
    addr_clients: HashMap<SocketAddr, ConnectedClient>,
    client_addrs: HashMap<ClientId, SocketAddr>,
    pending_action_broadcasts: Vec<(ClientId, PlayerAction)>,
}

struct ConnectedClient {
    id: ClientId,
    last_processed_action: u16,
    pending_messages: VecDeque<UdpMsgToClient>,
}

impl ConnectedClients {
    pub fn insert(
        &mut self,
        id: ClientId,
        last_processed_action: u16,
        addr: SocketAddr,
        entity: Entity,
    ) {
        self.client_entities.insert(id, entity);
        self.client_addrs.insert(id, addr);
        self.addr_clients.insert(
            addr,
            ConnectedClient {
                id,
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
                    .push_back(UdpMsgToClient::PlayerConnected { id })
            });
    }

    pub fn broadcast_action(&mut self, client_id: ClientId, action: PlayerAction) {
        self.pending_action_broadcasts.push((client_id, action));
    }

    pub fn update_last_msg(
        &mut self,
        last_processed_msg: u16,
        addr: &SocketAddr,
    ) -> Option<ClientId> {
        self.addr_clients.get_mut(addr).map(|client| {
            client.last_processed_action = last_processed_msg;
            client.id
        })
    }

    pub fn flush_pending_messages(
        &mut self,
        udp: &mut MultiUdpCommunicator<UdpMsgToClient, UdpMsgToServer>,
    ) {
        let mut iter = udp.iter_mut();
        while let Some(mut com) = iter.next() {
            let Some(ConnectedClient {
                id,
                last_processed_action,
                pending_messages,
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
                    action,
                });
            }
        }
        self.pending_action_broadcasts.clear();
    }

    pub fn get_client_entity(&self, client_id: &ClientId) -> Option<Entity> {
        self.client_entities.get(client_id).copied()
    }

    pub fn remove(&mut self, addr: SocketAddr) -> Option<Entity> {
        let client = self.addr_clients.remove(&addr)?;
        self.client_addrs.remove(&client.id).unwrap();

        info!("Removed client {:?}", client.id);

        // Broadcast disconnect
        for client in self.addr_clients.values_mut() {
            client
                .pending_messages
                .push_back(UdpMsgToClient::PlayerDisconnected { id: client.id })
        }

        self.client_entities.remove(&client.id)
    }
}

#[derive(Component, Deref, Debug)]
pub struct ClientAddr(pub SocketAddr);

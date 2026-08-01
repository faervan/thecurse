use std::sync::Arc;

use smol::{channel::SendError, lock::RwLock};

use crate::{clients::TcpClientChange, commands::TcpCommand, prelude::*};

pub struct ClientStore {
    command_sx: Sender<TcpCommand>,
    change_sx: Sender<TcpClientChange>,
    world: Arc<RwLock<String>>,
    next_id: usize,
    clients: HashMap<ClientId, Sender<TcpMsgToClient>>,
    // TODO! very ugly
    broadcast: Vec<Sender<TcpMsgToClient>>,
}

impl ClientStore {
    pub fn new(
        command_sx: Sender<TcpCommand>,
        change_sx: Sender<TcpClientChange>,
        world: Arc<RwLock<String>>,
    ) -> Self {
        Self {
            command_sx,
            change_sx,
            world,
            next_id: 0,
            clients: HashMap::new(),
            broadcast: vec![],
        }
    }

    pub async fn add_client(
        &mut self,
    ) -> Result<(ClientId, Receiver<TcpMsgToClient>), SendError<TcpMsgToClient>> {
        let (sx, rx) = smol::channel::unbounded();
        let id = ClientId(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);

        sx.send(TcpMsgToClient::ConnectionAccepted {
            client_id: id,
            world: self.world.read_arc().await.clone(),
        })
        .await?;
        self.broadcast(&TcpMsgToClient::ClientConnected(id)).await?;

        self.clients.insert(id, sx.clone());
        self.broadcast.push(sx);

        self.command_sx
            .send(TcpCommand::SpawnPlayer { client_id: id })
            .await
            .unwrap();

        self.change_sx
            .send(TcpClientChange::ClientConnect { id })
            .await
            .unwrap();

        Ok((id, rx))
    }

    pub async fn remove_client(&mut self, id: &ClientId) -> Result<(), SendError<TcpMsgToClient>> {
        self.clients.remove(id);
        self.change_sx
            .send(TcpClientChange::ClientDisconnect { id: *id })
            .await
            .unwrap();
        self.broadcast = self.clients.values().cloned().collect();
        self.broadcast(&TcpMsgToClient::ClientDisconnected(*id))
            .await?;
        Ok(())
    }

    #[inline(always)]
    pub async fn broadcast(&self, msg: &TcpMsgToClient) -> Result<(), SendError<TcpMsgToClient>> {
        for sx in &self.broadcast {
            sx.send(msg.clone()).await?;
        }
        Ok(())
    }
}

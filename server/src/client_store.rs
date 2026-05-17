use smol::channel::SendError;

use crate::prelude::*;

pub struct ClientStore {
    world: String,
    next_id: usize,
    clients: HashMap<ClientId, Sender<TcpMsgToClient>>,
    broadcast: Vec<Sender<TcpMsgToClient>>,
}

impl ClientStore {
    pub fn new(world: String) -> Self {
        Self {
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
            clients: self.clients.keys().copied().collect(),
            client_id: id,
            world: self.world.clone(),
        })
        .await?;
        self.broadcast(&TcpMsgToClient::ClientConnected(id)).await?;

        self.clients.insert(id, sx.clone());
        self.broadcast.push(sx);

        Ok((id, rx))
    }

    pub async fn remove_client(&mut self, id: &ClientId) -> Result<(), SendError<TcpMsgToClient>> {
        self.clients.remove(id);
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

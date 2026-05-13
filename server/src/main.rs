use std::{net::SocketAddr, sync::Arc};

use bevy::{
    app::{PanicHandlerPlugin, ScheduleRunnerPlugin, TerminalCtrlCHandlerPlugin},
    diagnostic::{DiagnosticsPlugin, FrameCountPlugin},
    gltf::GltfPlugin,
    log::LogPlugin,
    scene::ScenePlugin,
    state::app::StatesPlugin,
    tasks::AsyncComputeTaskPool,
    time::TimePlugin,
};
use futures::{AsyncReadExt as _, FutureExt as _, select};
use smol::{
    channel::SendError,
    io::AsyncWriteExt as _,
    lock::RwLock,
    net::{TcpListener, TcpStream},
    pin,
};
use thecurse_core::{
    networking::{ClientId, TcpMsgToClient, TcpMsgToServer, io_util::TheCurseReadWriteExt},
    prelude::*,
};

fn main() -> AppExit {
    let mut app = App::new();

    app.add_plugins((
        PanicHandlerPlugin,
        LogPlugin::default(),
        TaskPoolPlugin::default(),
        FrameCountPlugin,
        TimePlugin,
        TransformPlugin,
        DiagnosticsPlugin,
        ScheduleRunnerPlugin::default(),
        TerminalCtrlCHandlerPlugin,
        AssetPlugin::default(),
        ScenePlugin,
        GltfPlugin::default(),
        StatesPlugin,
    ));

    app.add_systems(Startup, setup);

    app.run()
}

fn setup() {
    let pool = AsyncComputeTaskPool::get();
    pool.spawn(handle_send_to_server()).detach();
}

struct ClientStore {
    next_id: usize,
    clients: HashMap<ClientId, Sender<TcpMsgToClient>>,
    broadcast: Vec<Sender<TcpMsgToClient>>,
}

impl ClientStore {
    fn new() -> Self {
        Self {
            next_id: 0,
            clients: HashMap::new(),
            broadcast: vec![],
        }
    }

    async fn add_client(
        &mut self,
    ) -> Result<(ClientId, Receiver<TcpMsgToClient>), SendError<TcpMsgToClient>> {
        let (sx, rx) = smol::channel::unbounded();
        let id = ClientId(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);

        sx.send(TcpMsgToClient::ConnectionAccepted {
            clients: self.clients.keys().copied().collect(),
            client_id: id,
        })
        .await?;
        self.broadcast(&TcpMsgToClient::ClientConnected(id)).await?;

        self.clients.insert(id, sx.clone());
        self.broadcast.push(sx);

        Ok((id, rx))
    }

    async fn remove_client(&mut self, id: &ClientId) -> Result<(), SendError<TcpMsgToClient>> {
        self.clients.remove(id);
        self.broadcast = self.clients.values().cloned().collect();
        self.broadcast(&TcpMsgToClient::ClientDisconnected(*id))
            .await?;
        Ok(())
    }

    #[inline(always)]
    async fn broadcast(&self, msg: &TcpMsgToClient) -> Result<(), SendError<TcpMsgToClient>> {
        for sx in &self.broadcast {
            sx.send(msg.clone()).await?;
        }
        Ok(())
    }
}

async fn handle_send_to_server() -> Result<(), TcpHandlerError> {
    let listener = TcpListener::bind("127.0.0.1:7189").await?;
    let store = Arc::new(RwLock::new(ClientStore::new()));

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let pool = AsyncComputeTaskPool::get();
                pool.spawn(handle_client(stream, addr, store.clone()))
                    .detach();
            }
            Err(e) => error!("couldn't get client: {e:?}"),
        }
    }
}

async fn handle_client(
    mut stream: TcpStream,
    addr: SocketAddr,
    client_store: Arc<RwLock<ClientStore>>,
) {
    info!("new client: {:?}", addr);
    let mut len_buf = [0_u8; 4];
    let mut buf = vec![];
    let (client_id, events_rx) = client_store.write().await.add_client().await.unwrap();
    loop {
        let read_future = stream.read_exact(&mut len_buf).fuse();
        pin!(read_future);

        let events_future = events_rx.recv().fuse();
        pin!(events_future);
        select! {
            result = read_future => {
                if result.is_err() {
                    break;
                }
                let msg = match TcpMsgToServer::read_from_with_len(&mut stream, &len_buf, &mut buf).await {
                    Ok(msg) => msg,
                    Err(e) => {
                        warn!("Failed to handle message from {client_id:?}: {e}");
                        continue;
                    }
                };
                match handle_message(&client_store, &mut stream, &addr, &client_id, msg).await {
                    Ok(true) => break,
                    Ok(false) => {}
                    Err(e) => error!("{e}"),
                }
            }
            result = events_future => {
                let event = match result {
                    Ok(e) => e,
                    Err(e) => {
                        warn!("Failed to read events of client {client_id:?}, \
                            dropping client connection. Error: {e}");
                        break;
                    }
                };
                info!("Got a new event: {event:?}");
                if let Err(e) = event.write_to(&mut stream).await {
                    warn!(
                        "Failed to propagate event {event:#?} to client {client_id:?}: {e}"
                    );
                    break;
                }
            }
        }
    }
    client_store
        .write()
        .await
        .remove_client(&client_id)
        .await
        .unwrap();
    info!("Client {addr:?} disconnected");
}

#[inline(always)]
/// Returns `Ok(true)` when the client disconnected
async fn handle_message(
    store: &Arc<RwLock<ClientStore>>,
    stream: &mut TcpStream,
    addr: &SocketAddr,
    client_id: &ClientId,
    msg: TcpMsgToServer,
) -> Result<bool, TcpHandlerError> {
    match msg {
        TcpMsgToServer::Connect => {
            info!("Client {addr:?} connected formally");
        }
        TcpMsgToServer::Message(msg) => {
            info!("Client {addr:?} send message: {msg}");
            store
                .read()
                .await
                .broadcast(&TcpMsgToClient::Message {
                    sender: *client_id,
                    message: msg,
                })
                .await?;
        }
        TcpMsgToServer::Disconnect => {
            info!("Client {addr:?} disconnected gracefully");
            stream.close().await?;
            return Ok(true);
        }
    }
    Ok(false)
}

#[derive(Debug, Error)]
enum TcpHandlerError {
    #[error("Network error: {0}")]
    NetworkError(#[from] smol::io::Error),
    #[error("Failed to serialize message: {0}")]
    SerializationFailed(#[from] postcard::Error),
    #[error("Failed to propagate message to clients: {0}")]
    PropagationFailed(#[from] smol::channel::SendError<TcpMsgToClient>),
}

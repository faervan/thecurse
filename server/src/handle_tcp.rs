use std::sync::Arc;

use bevy::tasks::AsyncComputeTaskPool;
use futures::{FutureExt as _, select};
use smol::{
    channel::unbounded,
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    lock::RwLock,
    net::{TcpListener, TcpStream},
    pin,
};

use crate::{
    commands::{TcpCommand, TcpCommandQueue},
    prelude::*,
    scene::{self, SerializedScene},
};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SerializedScene>();
    app.add_systems(Startup, (scene::setup, start_server));
    app.add_systems(
        FixedUpdate,
        scene::serialize_scene
            .pipe(scene::publish_scene)
            .run_if(scene::scene_requested),
    );
}

fn start_server(scene: Res<SerializedScene>, mut commands: Commands) {
    let pool = AsyncComputeTaskPool::get();
    let (sx, rx) = unbounded();
    commands.insert_resource(TcpCommandQueue { receiver: rx });
    pool.spawn(handle_send_to_server(
        sx,
        scene.notify.clone(),
        scene.world.clone(),
    ))
    .detach();
}

async fn handle_send_to_server(
    command_sx: Sender<TcpCommand>,
    notify: Arc<event_listener::Event>,
    world: Arc<RwLock<String>>,
) -> Result<(), TcpHandlerError> {
    let listener = TcpListener::bind("127.0.0.1:7189").await?;
    let store = Arc::new(RwLock::new(ClientStore::new(command_sx, world)));

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let pool = AsyncComputeTaskPool::get();
                pool.spawn(handle_client(stream, addr, notify.listen(), store.clone()))
                    .detach();
            }
            Err(e) => error!("couldn't get client: {e:?}"),
        }
    }
}

async fn handle_client(
    mut stream: TcpStream,
    addr: SocketAddr,
    notify: event_listener::EventListener,
    client_store: Arc<RwLock<ClientStore>>,
) {
    info!("new client: {:?}", addr);
    let mut len_buf = [0_u8; 4];
    let mut buf = vec![];
    notify.await;
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

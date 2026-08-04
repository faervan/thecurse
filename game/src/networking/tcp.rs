use bevy::tasks::AsyncComputeTaskPool;
use futures::{FutureExt as _, select};
use smol::{io::AsyncReadExt as _, net::TcpStream, pin};
use thecurse_core::networking::io_util::TheCurseIoError;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Game), setup_connection);
    app.add_systems(Update, add_clients.run_if(in_state(AppState::Game)));
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ServerConnection {
    #[reflect(ignore, default = "default_sx")]
    pub sender: Sender<TcpMsgToServer>,
    #[reflect(ignore, default = "default_rx")]
    pub receiver: Receiver<TcpMsgToClient>,
    pub client_id: Option<ClientId>,
    pub clients: HashMap<ClientId, Entity>,
}

fn default_sx() -> Sender<TcpMsgToServer> {
    smol::channel::unbounded().0
}

fn default_rx() -> Receiver<TcpMsgToClient> {
    smol::channel::unbounded().1
}

fn setup_connection(mut commands: Commands) {
    let (to_server_sx, to_server_rx) = smol::channel::unbounded();
    let (to_client_sx, to_client_rx) = smol::channel::unbounded();

    to_server_sx.send_blocking(TcpMsgToServer::Connect).unwrap();
    commands.insert_resource(ServerConnection {
        sender: to_server_sx,
        receiver: to_client_rx,
        client_id: None,
        clients: HashMap::new(),
    });

    let pool = AsyncComputeTaskPool::get();
    pool.spawn(handle_tcp(to_client_sx, to_server_rx)).detach();
}

async fn handle_tcp(
    sx: Sender<TcpMsgToClient>,
    rx: Receiver<TcpMsgToServer>,
) -> Result<(), TheCurseIoError> {
    let mut stream = TcpStream::connect("127.0.0.1:7189").await?;

    let mut len_buf = [0_u8; 4];
    let mut buf = vec![];
    loop {
        let from_server = stream.read_exact(&mut len_buf).fuse();
        pin!(from_server);

        let to_server = rx.recv().fuse();
        pin!(to_server);
        select! {
            result = from_server => {
                if result.is_err() {
                    break;
                }
                let msg = match TcpMsgToClient::read_from_with_len(&mut stream, &len_buf, &mut buf).await {
                    Ok(msg) => msg,
                    Err(e) => {
                        warn!(
                            "Got len of msg ({}) but msg reading failed: {e}",
                            u32::from_le_bytes(len_buf)
                        );
                        continue;
                    }
                };
                info!("Server send message: {msg:?}");
                if let Err(e) = sx.send(msg).await {
                    error!("Failed to forward message from server: {e}");
                }
            }
            result = to_server => {
                let msg = match result {
                    Ok(e) => e,
                    Err(e) => {
                        warn!("Failed to read message to server: {e}");
                        break;
                    }
                };
                if let Err(e) = msg.write_to(&mut stream).await {
                    warn!(
                        "Failed to propagate msg {msg:#?} to server: {e}"
                    );
                    break;
                }
            }
        }
    }

    Ok(())
}

fn add_clients(
    mut con: ResMut<ServerConnection>,
    clients: Query<(Entity, &ClientId), Added<ClientId>>,
) {
    con.clients
        .extend(clients.into_iter().map(|(e, id)| (*id, e)));
}

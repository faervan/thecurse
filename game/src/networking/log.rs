use bevy::scene::serde::SceneDeserializer;
use serde::de::DeserializeSeed;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Game), spawn_text);

    app.add_systems(
        Update,
        read_server_messages.run_if(in_state(AppState::Game)),
    );
}

#[derive(Component)]
struct TcpConLogText;

fn spawn_text(mut commands: Commands) {
    commands.spawn((
        Node {
            ..Default::default()
        },
        GameEntity,
        children![(
            TcpConLogText,
            Text::default(),
            TextFont::from_font_size(12.)
        )],
    ));
}

fn read_server_messages(
    mut con: ResMut<ServerConnection>,
    mut udp: ResMut<Udp>,
    query: Query<&mut Text, With<TcpConLogText>>,
    mut scenes: ResMut<Assets<DynamicScene>>,
    mut commands: Commands,
    mut recv_closed_log: Local<bool>,
    type_registry: Res<AppTypeRegistry>,
) {
    for mut text in query {
        loop {
            match con.receiver.try_recv() {
                Ok(msg) => {
                    text.push_str(&format!("{msg:?}\n"));
                    if let TcpMsgToClient::ConnectionAccepted {
                        ref world,
                        client_id,
                    } = msg
                    {
                        con.client_id = Some(client_id);
                        udp.write(UdpMsgToServer::Connect(client_id));
                        let result = SceneDeserializer {
                            type_registry: &type_registry.read(),
                        }
                        .deserialize(&mut ron::Deserializer::from_str(world).unwrap());
                        match result {
                            Ok(scene) => {
                                info!("success!");
                                let scene = scenes.add(scene);
                                commands.spawn(DynamicSceneRoot(scene));
                            }
                            Err(e) => error!("Scene deserialization failed: {e}"),
                        }
                    }
                }
                Err(smol::channel::TryRecvError::Empty) => break,
                Err(smol::channel::TryRecvError::Closed) => {
                    if !*recv_closed_log {
                        warn!("ServerConnection receiver closed");
                        *recv_closed_log = true;
                    }
                    break;
                }
            }
        }
    }
}

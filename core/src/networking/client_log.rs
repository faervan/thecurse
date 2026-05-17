use bevy::{reflect::TypeRegistry, scene::serde::SceneDeserializer};
use serde::de::DeserializeSeed;

use crate::{
    networking::{ServerConnection, TcpMsgToClient},
    prelude::*,
};

pub fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(OnEnter(game_state), spawn_text);

        app.add_systems(Update, read_server_messages.run_if(in_state(game_state)));
    }
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
    con: Res<ServerConnection>,
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
                    if let TcpMsgToClient::ConnectionAccepted { ref world, .. } = msg {
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
                    text.push_str(&format!("{msg:?}\n"))
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

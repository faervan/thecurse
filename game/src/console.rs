use bevy_console::{AddConsoleCommand as _, ConsoleCommand, ConsolePlugin};
use clap::Parser;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(ConsolePlugin);

    app.add_console_command::<ChatMessageCmd, _>(send_message);
    app.add_console_command::<TeleportCmd, _>(teleport);
}

/// Send a message to the global chat
#[derive(Parser, ConsoleCommand)]
#[command(name = "msg")]
struct ChatMessageCmd {
    /// Message to send
    msg: String,
}

fn send_message(mut log: ConsoleCommand<ChatMessageCmd>, con: Option<Res<ServerConnection>>) {
    if let Some(Ok(ChatMessageCmd { msg })) = log.take() {
        let Some(con) = con else {
            log.reply_failed("Not connected");
            return;
        };
        if let Err(e) = con.sender.send_blocking(TcpMsgToServer::Message(msg)) {
            log.reply_failed(format!("Failed to send message: {e}"));
            return;
        }

        log.ok();
    }
}

/// Teleport self
#[derive(Parser, ConsoleCommand)]
#[command(name = "tp")]
struct TeleportCmd {
    x: f32,
    y: f32,
    z: f32,
}

fn teleport(
    mut log: ConsoleCommand<TeleportCmd>,
    mut query: Query<&mut Transform, With<MainCharacter>>,
) {
    if let Some(Ok(TeleportCmd { x, y, z })) = log.take() {
        let mut transform = match query.single_mut() {
            Ok(t) => t,
            Err(e) => {
                log.reply_failed(format!("Failed to get MainCharacter: {e}"));
                return;
            }
        };

        transform.translation = vec3(x, y, z);
        log.ok();
    }
}

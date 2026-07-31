use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, apply_commands);
}

#[derive(Resource)]
pub struct TcpCommandQueue {
    pub receiver: Receiver<TcpCommand>,
}

pub enum TcpCommand {
    SpawnPlayer { client_id: ClientId },
}

fn apply_commands(queue: ResMut<TcpCommandQueue>, mut commands: Commands) {
    while let Ok(cmd) = queue.receiver.try_recv() {
        match cmd {
            TcpCommand::SpawnPlayer { client_id } => {
                commands.spawn((Player, Name::new(format!("Player #{}", client_id.0))));
            }
        }
    }
}

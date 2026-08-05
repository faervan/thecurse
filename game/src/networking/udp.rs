use thecurse_core::{networking::UDP_ADDR, utils::wrapping::wrapping_le};

use crate::{player::apply_action, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(AppState::Game), setup);

    app.add_systems(Update, read_udp_messages.run_if(in_state(AppState::Game)));
}

#[derive(Resource, Default)]
pub struct Udp {
    next_id: u16,
    action_cache: VecDeque<(u16, PlayerAction)>,
    com: UdpCommunicator<UdpMsgToServer, UdpMsgToClient>,
}

impl Udp {
    #[inline(always)]
    pub fn write(&mut self, msg: UdpMsgToServer) {
        self.com.write_ordered(msg);
    }

    pub fn write_action(&mut self, action: PlayerAction) {
        self.action_cache.push_back((self.next_id, action.clone()));
        let msg = UdpMsgToServer::Action {
            id: self.next_id,
            action,
        };
        self.com.write_ordered(msg);
        self.next_id = self.next_id.wrapping_add(1);
    }

    pub fn send(&mut self) {
        if self.com.last_send().elapsed().as_millis() > 200 {
            self.com.write_heartbeat();
        }
        self.com.send().unwrap();
    }

    pub fn recv_with<F>(&mut self, mut f: F)
    where
        F: FnMut(UdpMsgToClient),
    {
        self.com.recv();
        while let Some(msg) = self.com.read_ordered() {
            debug!("Received msg via UDP: {msg:?}");
            if let UdpMsgToClient::PlayerAction {
                last_processed_action,
                ..
            } = msg
            {
                while self
                    .action_cache
                    .pop_front_if(|m| wrapping_le(m.0, last_processed_action))
                    .is_some()
                {}
            }
            f(msg)
        }
    }

    pub fn disconnect(&mut self) {
        info!("Sending disconnect notification to server");
        self.write(UdpMsgToServer::Disconnect);
        self.com.send().unwrap();
    }
}

fn setup(mut commands: Commands) {
    let mut udp = Udp::default();
    udp.com.connect(UDP_ADDR).unwrap();
    commands.insert_resource(udp);
}

fn read_udp_messages(mut udp: ResMut<Udp>, con: Res<ServerConnection>, mut commands: Commands) {
    udp.recv_with(|msg| match msg {
        UdpMsgToClient::Connected { translation } => {
            let Some(client_id) = con.client_id else {
                error!("ClientId not initialized, not spawning player");
                return;
            };
            commands.spawn((
                MainCharacter,
                client_id,
                Transform::from_translation(Vec3::from_array(translation)),
            ));
        }
        UdpMsgToClient::PlayerConnected { id, translation } => {
            commands.spawn((
                Player,
                id,
                Transform::from_translation(Vec3::from_array(translation)),
            ));
        }
        UdpMsgToClient::PlayerDisconnected { id } => {
            if let Some(entity) = con.clients.get(&id) {
                debug!("Despawning player #{id:?} ({entity})");
                commands.entity(*entity).despawn();
            }
        }
        UdpMsgToClient::PlayerAction {
            client_id, action, ..
        } => {
            if let Some(entity) = con.clients.get(&client_id) {
                apply_action(action, *entity, &mut commands);
            }
        }
    });
    udp.send();
}

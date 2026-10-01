use bevy::ecs::schedule::ScheduleLabel;

use crate::prelude::*;

mod clients;
pub mod player;
pub mod prelude;
pub mod udp;

#[derive(Parser, Debug, Resource, Reflect)]
#[command(version, about)]
/// Server binary of "The Curse".
pub struct ServerSettings {
    #[arg(short, long, default_value_t = 7188)]
    /// UDP port to connect to.
    port_udp: u16,
}

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ServerBroadcast;

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct AfterServerBroadcast;

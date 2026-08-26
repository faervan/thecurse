use crate::prelude::*;

#[derive(Parser, Debug, Resource, Reflect)]
#[command(version, about)]
/// Game binary of "The Curse".
pub struct GameSettings {
    #[arg(short, long, default_value_t = 7188)]
    /// UDP port to connect to.
    pub port_udp: u16,

    #[arg(short = 'P', long, default_value_t = 7189)]
    /// TCP port to connect to.
    pub port_tcp: u16,

    #[arg(short, long)]
    #[cfg_attr(debug_assertions, arg(default_value = "0.0.0.0"))]
    #[cfg_attr(not(debug_assertions), arg(default_value = "72.61.104.16"))]
    /// Server address to connect to.
    pub addr: String,

    #[cfg(debug_assertions)]
    #[arg(long)]
    /// Setting this flag disables fake packet delay, corruption and drop.
    /// This is only available on debug builds.
    pub no_fake_unreliability: bool,
}

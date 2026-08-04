use crate::prelude::*;

mod log;
pub mod tcp;
pub mod udp;

pub fn plugin(app: &mut App) {
    app.add_plugins((udp::plugin, tcp::plugin, log::plugin));
}

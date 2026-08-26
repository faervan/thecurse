use crate::prelude::*;

pub const PROTOCOL_VERSION: u32 = 0;

pub const SERVER_TIMESTEP: Duration = Duration::from_millis(15625);

#[derive(ByteRepr, Debug)]
pub enum MsgToServer {
    Disconnect,
}

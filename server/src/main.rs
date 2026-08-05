use crate::prelude::*;
use bevy::{
    app::{PanicHandlerPlugin, TerminalCtrlCHandlerPlugin},
    ecs::schedule::ScheduleLabel,
    log::LogPlugin,
};

mod client_store;
mod clients;
mod debug;
mod handle_tcp;
mod handle_udp;
mod player;
mod prelude;
mod scene;

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ServerBroadcast;

fn main() -> AppExit {
    let mut app = App::new();

    app.add_schedule(Schedule::new(ServerBroadcast));

    app.add_systems(FixedLast, |world: &mut World, mut run: Local<bool>| {
        *run = !*run;
        // Fixed timestep is 40Hz, run ServerBroadcast at 20 Hz
        if *run {
            world.run_schedule(ServerBroadcast);
        }
    });

    app.add_plugins((
        MinimalPlugins,
        PanicHandlerPlugin,
        LogPlugin::default(),
        TransformPlugin,
        TerminalCtrlCHandlerPlugin,
    ));

    app.add_plugins((
        debug::plugin,
        handle_tcp::plugin,
        handle_udp::plugin,
        clients::plugin,
        player::plugin,
    ));

    app.insert_resource(Time::<Fixed>::from_hz(40.));

    app.run()
}

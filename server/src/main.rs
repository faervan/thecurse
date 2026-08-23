use crate::prelude::*;

use bevy::{
    app::{PanicHandlerPlugin, TerminalCtrlCHandlerPlugin},
    ecs::schedule::ScheduleLabel,
    log::LogPlugin,
    scene::ScenePlugin,
};
use clap::Parser;

mod client_store;
mod clients;
mod debug;
mod handle_tcp;
mod handle_udp;
mod player;
mod prelude;
mod scene;

#[derive(Parser, Debug, Resource, Reflect)]
#[command(version, about)]
/// Server binary of "The Curse".
pub struct ServerSettings {
    #[arg(short, long, default_value_t = 7188)]
    /// UDP port to connect to.
    port_udp: u16,

    #[arg(short = 'P', long, default_value_t = 7189)]
    /// TCP port to connect to.
    port_tcp: u16,
}

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ServerBroadcast;

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct AfterServerBroadcast;

fn main() -> AppExit {
    let settings = ServerSettings::parse();

    let mut app = App::new();

    app.add_schedule(Schedule::new(ServerBroadcast));
    app.add_schedule(Schedule::new(AfterServerBroadcast));

    app.insert_resource(Time::<Fixed>::from_duration(SERVER_TIMESTEP));
    app.insert_resource(settings);

    app.add_systems(FixedLast, |world: &mut World, mut run: Local<u8>| {
        *run += 1;
        // Fixed timestep is 64Hz, run ServerBroadcast at 16 Hz
        if *run == 4 {
            *run = 0;
            world.run_schedule(ServerBroadcast);
            world.run_schedule(AfterServerBroadcast);
        }
    });

    // Bevy basic plugins
    app.add_plugins((
        MinimalPlugins,
        PanicHandlerPlugin,
        LogPlugin::default(),
        TransformPlugin,
        TerminalCtrlCHandlerPlugin,
        AssetPlugin::default(),
        ScenePlugin,
    ));

    // Avian plugins
    app.add_plugins((
        PhysicsSchedulePlugin::default(),
        ColliderBackendPlugin::<Collider>::default(),
        ColliderHierarchyPlugin,
        ColliderTransformPlugin::default(),
        // since avian 0.6
        // ColliderTreePlugin
        BroadPhasePlugin::<()>::default(),
        // since avian 0.6
        // BvhBroadPhasePlugin
        NarrowPhasePlugin::<Collider>::default(),
        SolverPlugins::default(),
        JointPlugin,
        MassPropertyPlugin::default(),
        ForcePlugin,
        // Not needed on server?
        // SpatialQueryPlugin,
        PhysicsInterpolationPlugin::default(),
        PhysicsTransformPlugin::default(),
    ));

    app.add_plugins((
        debug::plugin,
        handle_tcp::plugin,
        handle_udp::plugin,
        clients::plugin,
        player::plugin,
    ));

    app.run()
}

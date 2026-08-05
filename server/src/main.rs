use crate::prelude::*;
use bevy::{
    app::{PanicHandlerPlugin, TerminalCtrlCHandlerPlugin},
    ecs::schedule::ScheduleLabel,
    log::LogPlugin,
    scene::ScenePlugin,
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

    app.add_systems(FixedLast, |world: &mut World, mut run: Local<u8>| {
        *run += 1;
        // Fixed timestep is 64Hz, run ServerBroadcast at 16 Hz
        if *run == 4 {
            *run = 0;
            world.run_schedule(ServerBroadcast);
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

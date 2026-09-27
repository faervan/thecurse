use bevy::{
    app::{PanicHandlerPlugin, TerminalCtrlCHandlerPlugin},
    log::LogPlugin,
    scene::ScenePlugin,
};
use dreamgame_server::prelude::*;

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
        asset_plugin(),
        ScenePlugin,
    ));

    // Avian plugins
    app.add_plugins((
        PhysicsSchedulePlugin::default(),
        MassPropertyPlugin::default(),
        ForcePlugin,
        ColliderHierarchyPlugin,
        ColliderTransformPlugin::default(),
        // Default collider
        ColliderBackendPlugin::<Collider>::default(),
        ColliderTreePlugin::<Collider>::default(),
        NarrowPhasePlugin::<Collider>::default(),
        //
        SolverPlugins::default(),
        //
        BroadPhaseCorePlugin,
        BvhBroadPhasePlugin::<()>::default(),
        JointPlugin,
        // Not needed on server?
        // SpatialQueryPlugin,
        PhysicsTransformPlugin::default(),
        PhysicsInterpolationPlugin::default(),
    ));

    app.add_plugins((dreamgame_server::udp::plugin));

    app.run()
}

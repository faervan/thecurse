use crate::prelude::*;
use bevy::{
    app::{PanicHandlerPlugin, TerminalCtrlCHandlerPlugin},
    log::LogPlugin,
};

mod client_store;
mod handle_tcp;
mod handle_udp;
mod prelude;
mod scene;

fn main() -> AppExit {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        PanicHandlerPlugin,
        LogPlugin::default(),
        TransformPlugin,
        TerminalCtrlCHandlerPlugin,
    ));

    app.add_plugins((handle_tcp::plugin, handle_udp::plugin));

    app.insert_resource(Time::<Fixed>::from_hz(10.));

    app.run()
}

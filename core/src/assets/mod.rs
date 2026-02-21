use crate::prelude::*;

mod loader;
pub use loader::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(loader::plugin);
}

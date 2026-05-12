use bevy::asset::{load_internal_asset, uuid::Uuid};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    load_internal_asset!(
        app,
        Handle::Uuid(Uuid::new_v4(), PhantomData),
        "../../assets/shaders/utils/gradient_noise.wgsl",
        Shader::from_wgsl
    );
    load_internal_asset!(
        app,
        Handle::Uuid(Uuid::new_v4(), PhantomData),
        "../../assets/shaders/utils/cubic_bezier.wgsl",
        Shader::from_wgsl
    );
}

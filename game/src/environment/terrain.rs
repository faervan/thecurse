use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(MaterialPlugin::<
        ExtendedMaterial<StandardMaterial, TerrainMaterial>,
    >::default());
}

#[derive(AsBindGroup, Asset, TypePath, Clone, Copy)]
struct TerrainMaterial {}

impl MaterialExtension for TerrainMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        "shaders/ground.wgsl".into()
    }
}

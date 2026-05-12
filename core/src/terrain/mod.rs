use bevy::{
    mesh::PlaneMeshBuilder,
    pbr::{ExtendedMaterial, MaterialExtension},
    render::render_resource::AsBindGroup,
};

use crate::prelude::*;

pub fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, TerrainMaterial>,
        >::default());

        app.add_systems(OnEnter(game_state), setup.run_if(|| false));
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, TerrainMaterial>>>,
) {
    commands.spawn((
        Name::new("Ground Terrain"),
        Mesh3d(meshes.add(PlaneMeshBuilder {
            plane: Plane3d::new(Vec3::Y, Vec2::splat(10.)),
            subdivisions: 30,
        })),
        MeshMaterial3d(materials.add(ExtendedMaterial {
            base: StandardMaterial::from_color(Color::srgb(0., 1., 0.)),
            extension: TerrainMaterial {},
        })),
        Transform::from_xyz(0., 0.5, 0.),
    ));
}

#[derive(AsBindGroup, Asset, TypePath, Clone, Copy)]
struct TerrainMaterial {}

impl MaterialExtension for TerrainMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        "shaders/ground.wgsl".into()
    }
}

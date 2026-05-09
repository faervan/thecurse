use bevy::render::render_resource::AsBindGroup;
use thecurse_core::spawn_camera;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(MaterialPlugin::<NoiseTestMaterial>::default());

    app.add_systems(OnEnter(AppState::Game), (setup.after(spawn_camera), hello));
}

fn hello() {
    info!("hello");
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<NoiseTestMaterial>>,
    camera: Single<Entity, With<CameraController>>,
) {
    commands.spawn((
        Name::new("NoiseTestPlane"),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::splat(0.5)))),
        MeshMaterial3d(materials.add(NoiseTestMaterial {})),
        Transform::from_xyz(0., 0., -2.),
        ChildOf(*camera),
    ));
}

#[derive(AsBindGroup, Asset, TypePath, Clone, Copy)]
struct NoiseTestMaterial {}

impl Material for NoiseTestMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/noise_test.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

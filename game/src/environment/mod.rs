use crate::prelude::*;

mod terrain;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(terrain::plugin);

    app.add_observer(on_rasterized_grid_obj_spawn);
    app.add_observer(on_rock_obj_spawn);
}

fn on_rasterized_grid_obj_spawn(
    event: On<Add, RasterizedGridObj>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    let mut data = vec![];
    let size = 100;
    for i in 0..size {
        for j in 0..size {
            if (i + j) % 2 == 0 {
                data.extend_from_slice(&[10, 10, 10, 255]);
            } else {
                data.extend_from_slice(&[30, 30, 30, 255]);
            }
        }
    }
    let mut image = Image::new(
        bevy::render::render_resource::Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Bgra8Unorm,
        bevy::asset::RenderAssetUsages::RENDER_WORLD | bevy::asset::RenderAssetUsages::MAIN_WORLD,
    );
    image.sampler = bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
        address_mode_u: bevy::image::ImageAddressMode::Repeat,
        address_mode_v: bevy::image::ImageAddressMode::Repeat,
        ..Default::default()
    });

    let mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.)));
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(image)),
        ..Default::default()
    });

    commands
        .entity(event.entity)
        .try_insert((Mesh3d(mesh), MeshMaterial3d(material)));
}

fn on_rock_obj_spawn(
    event: On<Add, RockObj>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let mesh = meshes.add(Cuboid::new(5., 5., 5.));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.1),
        ..Default::default()
    });

    commands
        .entity(event.entity)
        .try_insert((Mesh3d(mesh), MeshMaterial3d(material)));
}

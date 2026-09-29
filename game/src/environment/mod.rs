use dreamgame_core::environment::RasterizedGridCollider;

use crate::prelude::*;

pub fn spawn_obj_scene(scene: In<Box<dyn Scene>>, mut commands: Commands) {
    commands.spawn_scene(scene.0);
}

pub fn rasterized_grid_obj_scene(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) -> Box<dyn Scene> {
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

    Box::new(bsn! {
        #RasterizedGridObj
        Mesh3d(mesh)
        MeshMaterial3d::<StandardMaterial>(material)
        Children [
            (
                @RasterizedGridCollider
            ),
            (
                point_light_obj_scene()
                Transform::from_xyz(5., 3., 3.)
            )
        ]
    })
}

pub fn point_light_obj_scene() -> Box<dyn Scene> {
    Box::new(bsn! {
        #Light
        PointLight {
            intensity: 1_000_000.,
            range: 50.,
            shadow_maps_enabled: true,
        }
    })
}

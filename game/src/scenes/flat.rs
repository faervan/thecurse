use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameScene::Flat), spawn);
}

fn spawn(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    commands.spawn((
        Name::new("Ground"),
        Transform::default(),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(images.add(image)),
            ..Default::default()
        })),
        children![(
            Name::new("Ground Collider"),
            CursorTargetSurface,
            PhysicsPickable,
            RigidBody::Static,
            Collider::cuboid(100., 1., 100.),
            CollisionLayers::new(GameLayer::ENVIRONMENT, GameLayer::all()),
            Transform::from_xyz(0., -0.5, 0.)
        )],
    ));

    commands.spawn((
        Name::new("Light"),
        Transform::from_xyz(5., 3., 3.),
        PointLight {
            intensity: 1_000_000.,
            range: 50.,
            shadows_enabled: true,
            ..Default::default()
        },
    ));

    commands.spawn((
        Name::new("Rock"),
        Obstacle,
        CursorTargetSurface,
        PhysicsPickable,
        Transform::from_xyz(10., 2.5, 10.),
        Mesh3d(meshes.add(Cuboid::new(5., 5., 5.))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.1),
            ..Default::default()
        })),
        RigidBody::Static,
        Collider::cuboid(5., 5., 5.),
        CollisionLayers::new(GameLayer::ENVIRONMENT, GameLayer::all()),
    ));
}

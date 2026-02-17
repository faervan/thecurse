use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameScene::Flat), spawn);
}

fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Ground"),
        Transform::default(),
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(50.)))),
        MeshMaterial3d(materials.add(StandardMaterial::from_color(Color::WHITE))),
        RigidBody::Static,
        Collider::cuboid(50., 1., 50.),
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
}

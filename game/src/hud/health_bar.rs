use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(MaterialPlugin::<HealthBarMaterial>::default());

    app.add_systems(
        Update,
        (update_health_bars, despawn_health_bars).run_if(in_state(AppState::Game)),
    );
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[component(on_add)]
pub struct ShowHealthBar(Option<Handle<HealthBarMaterial>>);

#[derive(Component, Reflect)]
#[reflect(Component)]
struct HealthBar;

impl ShowHealthBar {
    fn on_add(mut world: DeferredWorld, hook: HookContext) {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh = meshes.add(Plane3d::new(Vec3::Z, Vec2::new(0.5, 0.05)));
        let mut materials = world.resource_mut::<Assets<HealthBarMaterial>>();
        let material = materials.add(HealthBarMaterial { health_percent: 1. });

        world.get_mut::<Self>(hook.entity).unwrap().0 = Some(material.clone());
        world.commands().spawn((
            Name::new("Health Bar"),
            HealthBar,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            NotShadowCaster,
            NotShadowReceiver,
            Billboarded,
            Follow {
                target: hook.entity,
                offset: Vec3::new(0., 1.1, 0.),
            },
        ));
    }
}

fn update_health_bars(
    query: Query<(&ShowHealthBar, &Health), Changed<Health>>,
    mut materials: ResMut<Assets<HealthBarMaterial>>,
) {
    for (health_bar, health) in query {
        if let Some(handle) = health_bar.0.as_ref()
            && let Some(material) = materials.get_mut(handle)
        {
            material.health_percent = health.current / health.max;
        }
    }
}

fn despawn_health_bars(
    mut commands: Commands,
    query: Query<Entity, (With<HealthBar>, Without<Follow>)>,
) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct HealthBarMaterial {
    #[uniform(0)]
    health_percent: f32,
}

impl Material for HealthBarMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/health_bar.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

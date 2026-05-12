use bevy::{
    light::{NotShadowCaster, NotShadowReceiver},
    render::render_resource::AsBindGroup,
};

use crate::{prelude::*, utils::follow::Follow};

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_message::<DealDamage>();

        app.add_plugins(MaterialPlugin::<HealthBarMaterial>::default());

        app.add_systems(
            Update,
            apply_damage.run_if(in_state(game_state).and(on_message::<DealDamage>)),
        );

        app.add_systems(
            Update,
            (update_health_bars, despawn_health_bars).run_if(in_state(game_state)),
        );
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct DamageSource {
    pub entity: Entity,
    pub damage: f32,
    /// By default, the [`on_collision_deal_damage`] observer will insert any entity that was hit
    /// into the `ignore` set, so that any damage source can only deal damage once to each entity.
    pub ignore: EntityHashSet,
}

impl DamageSource {
    pub fn new(entity: Entity, damage: f32) -> Self {
        Self {
            entity,
            damage,
            ignore: EntityHashSet::new(),
        }
    }
}

#[derive(Message, Debug)]
pub struct DealDamage {
    pub target: Entity,
    pub source: Entity,
    pub amount: f32,
}

pub fn on_collision_deal_damage(
    event: On<CollisionStart>,
    mut damage: MessageWriter<DealDamage>,
    mut sources: Query<&mut DamageSource>,
) {
    let Ok(mut source) = sources.get_mut(event.collider1) else {
        return;
    };
    if source.entity == event.collider2 || source.ignore.contains(&event.collider2) {
        return;
    }
    source.ignore.insert(event.collider2);
    damage.write(DealDamage {
        target: event.collider2,
        source: source.entity,
        amount: source.damage,
    });
}

#[macro_export]
macro_rules! on_collision_deal_damage_and {
    ($params:ty, |$e:ident, $s:ident, $p:ident| $apply:block) => {
        |event: On<CollisionStart>,
         mut damage: MessageWriter<DealDamage>,
         mut sources: Query<&mut DamageSource>,
         params: $params| {
            let Ok(mut source) = sources.get_mut(event.collider1) else {
                return;
            };
            if source.entity == event.collider2 || source.ignore.contains(&event.collider2) {
                return;
            }
            let $e = &event;
            let $s = &mut source;
            let $p = params;
            $apply
            source.ignore.insert(event.collider2);
            damage.write(DealDamage {
                target: event.collider2,
                source: source.entity,
                amount: source.damage,
            });
        }
    };
}

fn apply_damage(
    mut damage_reader: MessageReader<DealDamage>,
    mut query: Query<&mut Health>,
    mut commands: Commands,
) {
    for damage in damage_reader.read() {
        if let Ok(mut health) = query.get_mut(damage.target) {
            health.current -= damage.amount;
            if health.current <= 0. {
                commands.entity(damage.target).despawn();
                debug!("Entity {} was killed by {}", damage.target, damage.source);
            }
        }
    }
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

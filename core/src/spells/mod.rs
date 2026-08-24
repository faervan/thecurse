#[cfg(feature = "game")]
use bevy::{
    pbr::decal::{ForwardDecal, ForwardDecalMaterial, ForwardDecalMaterialExt},
    render::render_resource::AsBindGroup,
};

use crate::prelude::*;

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_message::<SpawnSpellVoid>();

        #[cfg(feature = "game")]
        app.add_plugins(MaterialPlugin::<ForwardDecalMaterial<VoidMaterial>>::default());

        app.add_systems(
            Update,
            (
                spawn_voids.run_if(on_message::<SpawnSpellVoid>),
                drive_void_spells,
            )
                .run_if(in_state(game_state)),
        );
    }
}

#[derive(Message)]
/// TODO! Should be generic spell cast message, not specific to void
pub struct SpawnSpellVoid {
    pub position: Vec3,
    pub caster: Entity,
}

#[derive(Component)]
struct VoidSpell {
    timer: Timer,
    caster: Entity,
    hitbox_spawned: bool,
    #[cfg(feature = "game")]
    material: Handle<ForwardDecalMaterial<VoidMaterial>>,
}

impl VoidSpell {
    fn new(
        secs: f32,
        #[cfg(feature = "game")] material: Handle<ForwardDecalMaterial<VoidMaterial>>,
        caster: Entity,
    ) -> Self {
        Self {
            timer: Timer::new(Duration::from_secs_f32(secs), TimerMode::Once),
            caster,
            hitbox_spawned: false,
            #[cfg(feature = "game")]
            material,
        }
    }
}

fn spawn_voids(
    mut spawner: MessageReader<SpawnSpellVoid>,
    mut commands: Commands,
    #[cfg(feature = "game")] mut materials: ResMut<Assets<ForwardDecalMaterial<VoidMaterial>>>,
) {
    for spawn in spawner.read() {
        #[cfg(feature = "game")]
        let material = materials.add(ForwardDecalMaterial {
            base: VoidMaterial::default(),
            extension: ForwardDecalMaterialExt {
                depth_fade_factor: 1.,
            },
        });
        commands.spawn((
            Name::new("Void Spell"),
            VoidSpell::new(
                1.,
                #[cfg(feature = "game")]
                material.clone(),
                spawn.caster,
            ),
            #[cfg(feature = "game")]
            (ForwardDecal, MeshMaterial3d(material)),
            Transform::from_translation(spawn.position + Vec3::Y * 0.1).with_scale(Vec3::splat(3.)),
            GameEntity,
        ));
    }
}

fn drive_void_spells(
    time: Res<Time>,
    mut commands: Commands,
    #[cfg(feature = "game")] mut materials: ResMut<Assets<ForwardDecalMaterial<VoidMaterial>>>,
    query: Query<(Entity, &mut VoidSpell)>,
) {
    for (entity, mut spell) in query {
        spell.timer.tick(time.delta());
        if spell.timer.just_finished() {
            commands.entity(entity).despawn();
        }

        let t = spell.timer.fraction();

        #[cfg(feature = "game")]
        if let Some(mut material) = materials.get_mut(&spell.material) {
            material.base.progress = t;
        }

        if t > 0.8 && !spell.hitbox_spawned {
            commands
                .entity(entity)
                .insert((
                    Collider::cylinder(0.4, 0.5),
                    CollisionEventsEnabled,
                    Sensor,
                    DamageSource::new(spell.caster, 4.),
                ))
                .observe(crate::on_collision_deal_damage_and!(
                    Commands,
                    |event, _source, params| {
                        let mut commands = params;
                        commands.entity(event.collider2).try_insert((
                            CrowdControlled(event.collider1),
                            CCPullTowards {
                                target: event.collider1,
                                intensity: 20.,
                            },
                        ));
                    }
                ));
        }
    }
}

#[cfg(feature = "game")]
#[derive(AsBindGroup, Asset, TypePath, Clone, Copy, Default)]
struct VoidMaterial {
    #[uniform(0)]
    progress: f32,
}

#[cfg(feature = "game")]
impl Material for VoidMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/spells/void.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

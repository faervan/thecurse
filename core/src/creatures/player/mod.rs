use crate::{creatures::BASIC_SWORD_DAMAGE, prelude::*};

mod actions;
pub use actions::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[component(on_add = <Player as IsCreature>::on_add)]
pub struct Player;

impl IsCreature for Player {
    const NAME: &str = "Player";
    const MAX_HEALTH: f32 = 20.;
    const GRAVITY_SCALE: f32 = 10.;

    fn collider() -> Collider {
        Collider::cuboid(0.5, 1.94, 0.2)
    }

    fn on_add_hook(mut world: DeferredWorld, this: Entity) {
        let mut cmds = world.commands();
        let collider = cmds
            .spawn((
                Name::new("PlayerMeleeAttackCollider"),
                Transform::from_xyz(0., 0., 1.),
                DamageSource::new(this, BASIC_SWORD_DAMAGE),
                CollisionEventsEnabled,
                CollisionLayers::new(GameLayer::WEAPON_MELEE, GameLayer::CREATURE),
                ChildOf(this),
            ))
            .observe(on_collision_deal_damage)
            .id();
        cmds.entity(this)
            .insert(PlayerMeleeAttackCollider(collider));
    }
}

use crate::{GameStateEntity, prelude::*};

pub mod behavior;
pub mod crowd_control;
pub mod goblin;
pub mod health;
pub mod player;

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_plugins((
            health::plugin(game_state),
            crowd_control::plugin(game_state),
            behavior::plugin(game_state),
            goblin::plugin(game_state),
        ));
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Creature;

pub const BASIC_SWORD_DAMAGE: f32 = 5.;

pub trait IsCreature: Component + Default {
    const NAME: &str;
    const MAX_HEALTH: f32;
    const GRAVITY_SCALE: f32;

    fn collider() -> Collider;

    fn bundle() -> impl Bundle {}

    #[allow(unused_variables)]
    fn on_add_hook(world: DeferredWorld, this: Entity) {}

    fn on_add(mut world: DeferredWorld, hook: HookContext) {
        let mut cmds = world.commands();
        let mut cmds = cmds.entity(hook.entity);
        cmds.try_insert_if_new((Self::bundle(), GameEntity, GameStateEntity))
            .try_insert_if_new(CreatureBundle::<Self>::default());
        Self::on_add_hook(world, hook.entity);
    }
}

#[derive(Bundle)]
pub struct CreatureBundle<C: IsCreature> {
    pub name: Name,
    pub health: Health,
    pub transform: Transform,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub layer: CollisionLayers,
    pub gravity_scale: GravityScale,
    pub locked_axes: LockedAxes,
    pub c: C,
    pub creature: Creature,
}

impl<C: IsCreature> Default for CreatureBundle<C> {
    fn default() -> Self {
        Self {
            name: Name::new(C::NAME),
            health: Health::new(C::MAX_HEALTH),
            transform: Transform::default(),
            rigid_body: RigidBody::Dynamic,
            collider: C::collider(),
            layer: CollisionLayers::new(
                GameLayer::CREATURE,
                GameLayer::DEFAULT | GameLayer::ENVIRONMENT | GameLayer::DAMAGE_SOURCE,
            ),
            gravity_scale: GravityScale(C::GRAVITY_SCALE),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            c: C::default(),
            creature: Creature,
        }
    }
}

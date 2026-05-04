use crate::prelude::*;

pub mod behavior;
pub mod goblin;
pub mod health;
pub mod weapon;

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_plugins((
            health::plugin(game_state),
            behavior::plugin(game_state),
            weapon::plugin,
            goblin::plugin(game_state),
        ));
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Creature;

#[derive(Bundle)]
pub struct CreatureBundle {
    pub name: Name,
    pub health: Health,
    pub scene: SceneRoot,
    pub transform: Transform,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub layer: CollisionLayers,
    pub gravity_scale: GravityScale,
    pub locked_axes: LockedAxes,
    pub creature: Creature,
}

impl Default for CreatureBundle {
    fn default() -> Self {
        Self {
            name: Name::default(),
            health: Health::new(20.),
            scene: SceneRoot::default(),
            transform: Transform::default(),
            rigid_body: RigidBody::Dynamic,
            collider: Collider::cuboid(0.8, 1.8, 0.3),
            layer: CollisionLayers::new(
                GameLayer::CREATURE,
                GameLayer::DEFAULT | GameLayer::ENVIRONMENT | GameLayer::DAMAGE_SOURCE,
            ),
            gravity_scale: GravityScale(10.),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            creature: Creature,
        }
    }
}

use crate::prelude::*;

pub mod behavior;
pub mod goblin;
pub mod health;

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_plugins((
            health::plugin(game_state),
            behavior::plugin(game_state),
            goblin::plugin(game_state),
        ));

        app.add_systems(OnEnter(game_state), test);
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
            health: Health(20.),
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

fn test(mut commands: Commands) {
    commands.spawn((
        Name::new("Test dummy"),
        Health(20.),
        Transform::from_xyz(3., 1., 1.),
        Collider::cuboid(0.8, 2., 0.5),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        CollisionLayers::new(
            GameLayer::CREATURE,
            GameLayer::DEFAULT | GameLayer::ENVIRONMENT | GameLayer::DAMAGE_SOURCE,
        ),
    ));
}

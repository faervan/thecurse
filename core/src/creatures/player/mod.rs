use crate::prelude::*;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[component(on_add = <Player as IsCreature>::on_add)]
pub struct Player;

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Player)]
pub struct MainCharacter;

impl IsCreature for Player {
    const NAME: &str = "Player";
    const MAX_HEALTH: f32 = 20.;
    const GRAVITY_SCALE: f32 = 10.;

    fn collider() -> Collider {
        Collider::cuboid(0.5, 1.94, 0.2)
    }
}

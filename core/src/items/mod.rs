use ambassador::{Delegate, delegatable_trait};

use crate::prelude::*;

#[derive(Delegate, Reflect, Debug)]
#[delegate(ItemEffect)]
pub enum Item {
    KillAllGoblins(ItemKillAllGoblins),
    SpawnGoblin(ItemSpawnGoblin),
    SpawnTwoGoblins(ItemSpawnTwoGoblins),
    SpawnGoblinsRandom(ItemSpawnGoblinsRandom),
}

#[delegatable_trait]
pub trait ItemEffect {
    fn use_effect(&mut self, commands: &mut Commands);
}

#[derive(Reflect, Debug)]
pub struct ItemKillAllGoblins;

impl ItemEffect for ItemKillAllGoblins {
    fn use_effect(&mut self, commands: &mut Commands) {
        commands.run_system_cached(kill_all_goblins);
    }
}

fn kill_all_goblins(mut commands: Commands, query: Query<Entity, With<Goblin>>) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}

#[derive(Reflect, Debug)]
pub struct ItemSpawnGoblin;

impl ItemEffect for ItemSpawnGoblin {
    fn use_effect(&mut self, commands: &mut Commands) {
        commands.run_system_cached(spawn_goblin);
    }
}

fn spawn_goblin(mut commands: Commands, player: Query<&Transform, With<Player>>) {
    let Ok(transform) = player.single() else {
        warn!("Can't spawn goblin in front of player: MainCharacter not found");
        return;
    };
    let position = transform.translation + transform.rotation * Vec3::Z * 5.;
    commands.spawn((Goblin, Transform::from_translation(position)));
}

#[derive(Reflect, Debug)]
pub struct ItemSpawnTwoGoblins;

impl ItemEffect for ItemSpawnTwoGoblins {
    fn use_effect(&mut self, commands: &mut Commands) {
        commands.run_system_cached(spawn_goblin);
        commands.run_system_cached(spawn_goblin);
    }
}

#[derive(Reflect, Debug)]
pub struct ItemSpawnGoblinsRandom;

impl ItemEffect for ItemSpawnGoblinsRandom {
    fn use_effect(&mut self, commands: &mut Commands) {
        commands.run_system_cached(random_spawn_goblins);
    }
}

fn random_spawn_goblins(mut commands: Commands) {
    let mut rng = rand::rng();
    use rand::RngExt as _;
    for _ in 0..80 {
        let x = rng.random_range(-50_f32..50_f32);
        let z = rng.random_range(-50_f32..50_f32);
        commands.spawn((Goblin, Transform::from_translation(Vec3::new(x, 1., z))));
    }
}

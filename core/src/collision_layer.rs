use bitflags::bitflags;

use crate::prelude::*;

#[derive(Reflect, Clone, Copy, Debug, Default)]
pub struct GameLayer(u32);

bitflags! {
    impl GameLayer: u32 {
        const DEFAULT = 1 << 0;

        const ENVIRONMENT = 1 << 1;
        const ITEM = 1 << 2;
        const WEAPON_MELEE = 1 << 3;
        const PROJECTILE = 1 << 4;
        const DAMAGE_SOURCE = Self::WEAPON_MELEE.bits() | Self::PROJECTILE.bits();

        // Creatures
        const PLAYER = 1 << 5;
        const GOBLIN = 1 << 6;
        const CREATURE = Self::PLAYER.bits() | Self::GOBLIN.bits();
    }
}

impl PhysicsLayer for GameLayer {
    fn to_bits(&self) -> u32 {
        self.bits()
    }
    fn all_bits() -> u32 {
        Self::all().bits()
    }
}

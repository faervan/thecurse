use thecurse_core::items;

use crate::prelude::*;

pub(super) fn plguin(app: &mut App) {
    app.init_resource::<Inventory>();

    app.add_systems(
        Update,
        use_hotbar_item_on_keypress.run_if(in_state(AppState::Game)),
    );
}

#[derive(Resource, Reflect, Debug)]
struct Inventory {
    hotbar: [Option<Item>; 9],
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            hotbar: [
                Some(Item::KillAllGoblins(items::ItemKillAllGoblins)),
                Some(Item::SpawnGoblin(items::ItemSpawnGoblin)),
                Some(Item::SpawnTwoGoblins(items::ItemSpawnTwoGoblins)),
                Some(Item::SpawnGoblinsRandom(items::ItemSpawnGoblinsRandom)),
                Some(Item::RespawnPlayer(items::ItemRespawnPlayer)),
                None,
                None,
                None,
                None,
            ],
        }
    }
}

fn use_hotbar_item_on_keypress(
    input: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<Inventory>,
    mut commands: Commands,
) {
    for (index, key) in [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ]
    .into_iter()
    .enumerate()
    {
        if input.just_pressed(key)
            && let Some(item) = &mut inventory.hotbar[index]
        {
            debug!("Using item from hotbar {}: {item:?}", index + 1);
            item.use_effect(&mut commands);
        }
    }
}

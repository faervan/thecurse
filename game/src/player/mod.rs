use crate::prelude::*;

mod actions;
mod asset_loading;
pub mod cursor_target;
mod inventory;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((actions::plugin, inventory::plguin, cursor_target::plugin));

    app.load_assets_with(asset_loading::load_player_assets);

    app.add_observer(on_player_spawn);
}

#[derive(Resource, TypePath)]
pub struct PlayerCharacterHandle {
    scene: Handle<Scene>,
    idle: AnimationNodeIndex,
    running: AnimationNodeIndex,
    jumping: AnimationNodeIndex,
    falling: AnimationNodeIndex,
    attack: AnimationNodeIndex,
    attack_bottom: AnimationNodeIndex,
}

impl GltfAssetPath for PlayerCharacterHandle {
    const PATH: &'static str = "models/Player.glb";
}

fn on_player_spawn(
    event: On<Add, Player>,
    character: Res<PlayerCharacterHandle>,
    mut commands: Commands,
) {
    commands
        .entity(event.entity)
        .try_insert((SceneRoot(character.scene.clone()), ShowHealthBar::default()))
        .observe(on_ready_insert_child_pointer::<GltfAnimationTarget>)
        .observe(on_ready_insert_child_pointer::<WeaponSocketHandle>);
}

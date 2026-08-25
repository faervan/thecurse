use crate::{player::actions::CachedPlayerAction, prelude::*};

mod actions;
pub use actions::apply_action;
use thecurse_core::{
    creatures::player::{AerialState, AttackState, MovementState},
    utils::wrapping::wrapping_le,
};

mod asset_loading;
pub mod cursor_target;
mod inventory;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((actions::plugin, inventory::plguin, cursor_target::plugin));

    app.load_assets_with(asset_loading::load_player_assets);

    app.add_observer(on_player_spawn);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Player, AerialState, MovementState, AttackState)]
pub struct MainCharacter {
    next_id: u16,
    action_cache: VecDeque<(u16, CachedPlayerAction)>,
    pub new_messages: Vec<UdpMsgToServer>,
    /// Deviation of the predicted translation from the authoritative server translation that has
    /// not been corrected yet.
    authoritative_translation: Vec3,
    predicted_movement: Vec3,
    correction: Vec3,
    correction_progress: f32,
}

impl MainCharacter {
    pub fn new(translation: Vec3) -> Self {
        Self {
            next_id: 0,
            action_cache: VecDeque::new(),
            new_messages: vec![],
            authoritative_translation: translation,
            predicted_movement: Vec3::ZERO,
            correction: Vec3::ZERO,
            correction_progress: 1.,
        }
    }

    pub fn add_action(&mut self, action: CachedPlayerAction) {
        self.new_messages.push(UdpMsgToServer::Action {
            id: self.next_id,
            action: match &action {
                CachedPlayerAction::Movement { action, .. } => action.clone(),
            },
        });
        self.action_cache.push_back((self.next_id, action));
        self.next_id = self.next_id.wrapping_add(1);
    }

    pub fn handle_action(&mut self, action: PlayerActionBroadcast, last_processed_action: u16) {
        while self
            .action_cache
            .pop_front_if(|(action_id, _)| wrapping_le(*action_id, last_processed_action))
            .is_some()
        {}
        match action {
            PlayerActionBroadcast::Movement { destination, .. } => {
                self.authoritative_translation = Vec3::from_array(destination);
                self.predicted_movement = self
                    .action_cache
                    .iter()
                    .map(|(_, a)| {
                        let CachedPlayerAction::Movement { motion, .. } = a;
                        motion
                    })
                    .sum::<Vec3>()
                    - self.correction * (1. - self.correction_progress);
                self.correction_progress = 0.;
                debug!("Received authoritative position: {destination:?}",);
            }
            PlayerActionBroadcast::Attack { .. } => {}
        }
    }
}

#[derive(Resource, TypePath)]
pub struct PlayerCharacterHandle {
    scene: Handle<WorldAsset>,
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
        .try_insert((
            WorldAssetRoot(character.scene.clone()),
            ShowHealthBar::default(),
        ))
        .observe(on_ready_insert_child_pointer::<GltfAnimationTarget>)
        .observe(on_ready_insert_child_pointer::<WeaponSocketHandle>);
}

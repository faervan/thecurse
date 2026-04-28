use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::Game),
        spawn_player.after(thecurse_core::spawn_camera),
    );
    app.add_systems(Update, player_input);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct PlayerAnimationTarget(Entity);

fn spawn_player(mut spawner: MessageWriter<SpawnPlayer>) {
    spawner.write(SpawnPlayer {
        position: Vec3::splat(1.),
    });
}

fn player_input(input: Res<ButtonInput<KeyCode>>) {}

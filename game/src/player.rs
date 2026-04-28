use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::Game),
        spawn_player.after(thecurse_core::spawn_camera),
    );
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct PlayerAnimationTarget(Entity);

fn spawn_player(mut spawner: MessageWriter<SpawnPlayer>) {
    debug!("Spawn Player");
    spawner.write(SpawnPlayer {
        position: Vec3::splat(1.),
    });
}

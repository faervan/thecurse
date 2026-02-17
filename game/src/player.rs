use thecurse_core::{
    CameraControllerAnchor,
    utils::{GltfAssetPath, GltfLoadingHandle},
};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::Game),
        spawn_player.after(thecurse_core::spawn_camera),
    );
    app.add_systems(Update, player_input);
}

#[derive(Resource, TypePath)]
struct PlayerCharacterHandle {
    scene: Handle<Scene>,
    jumping: AnimationNodeIndex,
}

impl GltfAssetPath for PlayerCharacterHandle {
    const PATH: &'static str = "models/Player.glb";
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct PlayerAnimationTarget(Entity);

fn spawn_player(
    mut commands: Commands,
    ankor: Single<Entity, With<CameraControllerAnchor>>,
    player_handle: Res<PlayerCharacterHandle>,
) {
    let id = commands
        .spawn((
            Name::new("Player"),
            SceneRoot(player_handle.scene.clone()),
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            Collider::cuboid(0.5, 2., 0.2),
            GravityScale(0.),
        ))
        .observe(
            |event: On<SceneInstanceReady>,
             mut commands: Commands,
             query: Query<(Option<&Children>, Has<AnimationPlayer>)>| {
                let mut current = vec![event.entity];
                'outer: loop {
                    for entity in std::mem::take(&mut current) {
                        if let Ok((children_maybe, has_player)) = query.get(entity) {
                            if has_player {
                                commands
                                    .entity(event.entity)
                                    .insert(PlayerAnimationTarget(entity));
                                break 'outer;
                            }
                            if let Some(children) = children_maybe {
                                current.extend(children);
                            }
                        }
                    }
                }
                // Remove this observer
                commands.entity(event.entity).remove::<ObservedBy>();
            },
        )
        .id();

    commands.entity(*ankor).insert(ChildOf(id));
}

fn player_input(input: Res<ButtonInput<KeyCode>>) {}

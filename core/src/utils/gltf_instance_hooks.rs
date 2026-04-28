use bevy::scene::SceneInstanceReady;

use crate::prelude::*;

#[derive(Component, Reflect, Debug, Deref, DerefMut)]
#[reflect(Component)]
pub struct GltfAnimationTarget(pub Entity);

pub fn on_ready_insert_animation_target(
    event: On<SceneInstanceReady>,
    mut commands: Commands,
    query: Query<(Option<&Children>, Has<AnimationPlayer>)>,
) {
    let mut current = vec![event.entity];

    // Search for an [`AnimationPlayer`] and assume the first one found is the armature we want
    'outer: loop {
        if current.is_empty() {
            break;
        }
        for entity in std::mem::take(&mut current) {
            if let Ok((children_maybe, has_player)) = query.get(entity) {
                if has_player {
                    commands
                        .entity(event.entity)
                        .insert(GltfAnimationTarget(entity));
                    break 'outer;
                }
                if let Some(children) = children_maybe {
                    current.extend(children);
                }
            }
        }
    }

    // Remove this observer
    commands.entity(event.observer()).despawn();
}

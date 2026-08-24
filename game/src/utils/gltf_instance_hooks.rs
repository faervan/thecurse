use bevy::world_serialization::WorldInstanceReady;

use crate::prelude::*;

#[derive(Component, Reflect, Debug, Deref, DerefMut)]
#[reflect(Component)]
pub struct GltfAnimationTarget(pub Entity);

impl ChildEntityPointer for GltfAnimationTarget {
    type Target = AnimationPlayer;
    fn new(entity: Entity) -> Self {
        Self(entity)
    }
}

/// A [`Component`] that holds the [`Entity`] of a child with some component
/// [`ChildEntityPointer::Target`].
/// Used e.g. to get the child entity of a gltf entity that holds the [`AnimationPlayer`].
pub trait ChildEntityPointer: Component {
    type Target: Component;

    fn new(entity: Entity) -> Self;
}

pub fn on_ready_insert_child_pointer<T: ChildEntityPointer>(
    event: On<WorldInstanceReady>,
    mut commands: Commands,
    query: Query<(Option<&Children>, Has<T::Target>)>,
) {
    let mut current = vec![event.entity];

    // Search for an [`AnimationPlayer`] and assume the first one found is the armature we want
    'outer: loop {
        if current.is_empty() {
            break;
        }
        for entity in std::mem::take(&mut current) {
            if let Ok((children_maybe, has_t)) = query.get(entity) {
                if has_t {
                    commands.entity(event.entity).insert(T::new(entity));
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

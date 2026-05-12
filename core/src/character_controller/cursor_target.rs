use bevy::picking::pointer::PointerInteraction;

use crate::prelude::*;

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.init_resource::<CursorTargetPosition>();
        app.init_resource::<CursorTargetHashSet>();

        app.add_systems(
            Update,
            update_cursor_target_position.run_if(in_state(game_state)),
        );
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[component(on_add, on_remove)]
/// TODO! Currently this approach is suboptimal because the target surfaces can be blocked from
/// being picked by non-target entities that are detected as being closer by the picking backend.
/// Hovering over this entity updates the characters [`CursorTargetPosition`].
pub struct CursorTargetSurface;

#[derive(Resource, Reflect, Debug, Default, Deref)]
#[reflect(Resource)]
pub struct CursorTargetPosition(Vec3);

impl CursorTargetSurface {
    fn on_add(mut world: DeferredWorld, hook: HookContext) {
        world
            .resource_mut::<CursorTargetHashSet>()
            .insert(hook.entity);
    }

    fn on_remove(mut world: DeferredWorld, hook: HookContext) {
        world
            .resource_mut::<CursorTargetHashSet>()
            .remove(&hook.entity);
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
struct CursorTargetHashSet(EntityHashSet);

fn update_cursor_target_position(
    mut cursor_target: ResMut<CursorTargetPosition>,
    target_entities: Res<CursorTargetHashSet>,
    events: Query<&PointerInteraction>,
) {
    debug!("{:?}", events.single().unwrap());
    if let Some(position) = events
        .single()
        .ok()
        .and_then(|interaction| interaction.get_nearest_hit())
        .filter(|(entity, _hit)| target_entities.contains(entity))
        .and_then(|(_entity, hit)| hit.position.zip(hit.normal))
        .and_then(|(pos, normal)| (normal.dot(Vec3::Y) > 0.8).then_some(pos))
    {
        cursor_target.0 = position;
    }
}

use bevy::picking::pointer::PointerInteraction;

use crate::prelude::*;

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.init_resource::<CursorTargetPosition>();

        app.add_systems(
            Update,
            update_cursor_target_position.run_if(in_state(game_state)),
        );
    }
}

#[derive(Resource, Reflect, Debug, Default, Deref)]
#[reflect(Resource)]
pub struct CursorTargetPosition(Vec3);

fn update_cursor_target_position(
    mut cursor_target: ResMut<CursorTargetPosition>,
    events: Query<&PointerInteraction>,
) {
    if let Some(position) = events
        .single()
        .ok()
        .and_then(|interaction| interaction.get_nearest_hit())
        .and_then(|(_entity, hit)| hit.position.zip(hit.normal))
        .and_then(|(pos, normal)| (normal.dot(Vec3::Y) > 0.8).then_some(pos))
    {
        cursor_target.0 = position;
    }
}

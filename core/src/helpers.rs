use crate::prelude::*;

/// Compare two `[f32; 3]`, allowing minimal deviation of `0.05`.
pub fn approx_eq(left: [f32; 3], right: [f32; 3]) -> bool {
    left.into_iter()
        .zip(right)
        .all(|(l, r)| (l - r).abs() < 0.05)
}

pub fn resource_in_state<R: Resource + Default>(state: impl States + Copy) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(OnEnter(state), |mut commands: Commands| {
            commands.insert_resource(R::default());
        });
        app.add_systems(OnExit(state), |mut commands: Commands| {
            commands.remove_resource::<R>();
        });
    }
}

use crate::prelude::*;

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_message::<DealDamage>();

        app.add_systems(
            Update,
            apply_damage.run_if(in_state(game_state).and(on_message::<DealDamage>)),
        );
    }
}

#[derive(Component, Reflect, Debug, Deref, DerefMut)]
#[reflect(Component)]
pub struct Health(pub f32);

#[derive(Message, Debug)]
pub struct DealDamage {
    pub target: Entity,
    pub amount: f32,
}

fn apply_damage(
    mut damage_reader: MessageReader<DealDamage>,
    mut query: Query<&mut Health>,
    mut commands: Commands,
) {
    for damage in damage_reader.read() {
        if let Ok(mut health) = query.get_mut(damage.target) {
            **health -= damage.amount;
            if health.is_sign_negative() {
                commands.entity(damage.target).despawn();
                debug!("Entity {} was killed", damage.target);
            }
        }
    }
}

use crate::prelude::*;

pub(super) fn plugin<STATE: States + Copy>(game_state: STATE) -> impl Plugin {
    move |app: &mut App| {
        app.add_systems(Update, pull_towards.run_if(in_state(game_state)));
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = CrowdController)]
pub struct CrowdControlled(pub Entity);

#[derive(Component, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = CrowdControlled)]
pub struct CrowdController(Vec<Entity>);

#[derive(Component, Reflect)]
#[reflect(Component)]
/// Gets automatically removed when the target entity is despawned or does not have a [`Transform`]
pub struct CCPullTowards {
    pub target: Entity,
    pub intensity: f32,
}

fn pull_towards(
    mut commands: Commands,
    query: Query<(Entity, &mut LinearVelocity, &Transform, &CCPullTowards)>,
    targets: Query<&Transform>,
) {
    for (entity, mut velocity, transform, cc) in query {
        let Ok(target_transform) = targets.get(cc.target) else {
            commands.entity(entity).remove::<CCPullTowards>();
            continue;
        };
        **velocity = (target_transform.translation - transform.translation) * cc.intensity;
    }
}

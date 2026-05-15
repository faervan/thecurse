use crate::prelude::*;

pub struct FollowUtilPlugin<STATE> {
    state: STATE,
}

impl<STATE> FollowUtilPlugin<STATE> {
    pub fn new(state: STATE) -> Self {
        Self { state }
    }
}

impl<STATE> Plugin for FollowUtilPlugin<STATE>
where
    STATE: States,
{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, follow.run_if(in_state(self.state.clone())));
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = FollowedBy)]
/// This entity will "follow" the [`Follow.target`] entity.
/// Following means its [`Transform.translation`] will by synced with the [`target`] entity.
/// Entities that have this component cannot be a followed target themselves.
pub struct Follow {
    #[relationship]
    /// The id of the [`Entity`] that is being followed
    pub target: Entity,
    /// The offset from the target entity. Any rotation will be disregarded.
    pub offset: Vec3,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = Follow)]
pub struct FollowedBy(Vec<Entity>);

fn follow(
    followers: Query<(&mut Transform, &Follow)>,
    followed: Query<&Transform, Without<Follow>>,
) {
    for (mut transform, follow) in followers {
        if let Ok(target_transform) = followed.get(follow.target) {
            transform.translation = target_transform.translation + follow.offset;
        }
    }
}

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

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct DamageSource {
    pub entity: Entity,
    pub damage: f32,
    /// By default, the [`on_collision_deal_damage`] observer will insert any entity that was hit
    /// into the `ignore` set, so that any damage source can only deal damage once to each entity.
    pub ignore: EntityHashSet,
}

impl DamageSource {
    pub fn new(entity: Entity, damage: f32) -> Self {
        Self {
            entity,
            damage,
            ignore: EntityHashSet::new(),
        }
    }
}

#[derive(Message, Debug)]
pub struct DealDamage {
    pub target: Entity,
    pub source: Entity,
    pub amount: f32,
}

pub fn on_collision_deal_damage(
    event: On<CollisionStart>,
    mut damage: MessageWriter<DealDamage>,
    mut sources: Query<&mut DamageSource>,
) {
    let Ok(mut source) = sources.get_mut(event.collider1) else {
        return;
    };
    if source.entity == event.collider2 || source.ignore.contains(&event.collider2) {
        return;
    }
    source.ignore.insert(event.collider2);
    damage.write(DealDamage {
        target: event.collider2,
        source: source.entity,
        amount: source.damage,
    });
}

#[macro_export]
macro_rules! on_collision_deal_damage_and {
    ($params:ty, |$e:ident, $s:ident, $p:ident| $apply:block) => {
        |event: On<CollisionStart>,
         mut damage: MessageWriter<DealDamage>,
         mut sources: Query<&mut DamageSource>,
         params: $params| {
            let Ok(mut source) = sources.get_mut(event.collider1) else {
                return;
            };
            if source.entity == event.collider2 || source.ignore.contains(&event.collider2) {
                return;
            }
            let $e = &event;
            let $s = &mut source;
            let $p = params;
            $apply
            source.ignore.insert(event.collider2);
            damage.write(DealDamage {
                target: event.collider2,
                source: source.entity,
                amount: source.damage,
            });
        }
    };
}

fn apply_damage(
    mut damage_reader: MessageReader<DealDamage>,
    mut query: Query<&mut Health>,
    mut commands: Commands,
) {
    for damage in damage_reader.read() {
        if let Ok(mut health) = query.get_mut(damage.target) {
            health.current -= damage.amount;
            if health.current <= 0. {
                commands.entity(damage.target).despawn();
                debug!("Entity {} was killed by {}", damage.target, damage.source);
            }
        }
    }
}

use std::{fmt::Debug, marker::PhantomData, time::Duration};

use bevy::{
    ecs::{component::Mutable, query::QueryData, system::ObserverSystem},
    prelude::*,
};

mod animation_impl;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                #[cfg(feature = "game")]
                animate::<&mut BackgroundColor>,
                #[cfg(feature = "game")]
                animate::<&mut BorderColor>,
                animate::<&mut Transform>,
            ),
        );
    }
}

#[derive(Component)]
pub struct Animation<Q: QueryData> {
    f: Box<dyn FnMut(Q::Item<'_, '_>, f32) + Send + Sync + 'static>,
    timer: Timer,
    _c: PhantomData<Q>,
}

impl<Q: QueryData> Animation<Q> {
    pub fn new<O, F>(f: F, duration_ms: u64, origin: O::Item<'_, '_>) -> Self
    where
        O: QueryData,
        F: IntoTransformationAnimation<Q, O>,
    {
        Self {
            f: Box::new(f.as_transform(origin)),
            timer: Timer::new(Duration::from_millis(duration_ms), TimerMode::Once),
            _c: PhantomData,
        }
    }
}

pub trait IntoTransformationAnimation<Q, O = <Q as QueryData>::ReadOnly>:
    Send + Sync + 'static
where
    Q: QueryData,
    O: QueryData,
{
    fn as_transform<'a, 'b>(
        &self,
        origin: O::Item<'a, 'b>,
    ) -> impl FnMut(Q::Item<'_, '_>, f32) + Send + Sync + 'static;
    fn reverse(&self, origin: O::Item<'_, '_>) -> Self;
}

/// Trigger an animation when [Pointer<E>] was fired.
/// See [pointer_events](https://docs.rs/bevy/latest/bevy/picking/events/fn.pointer_events.html)
pub fn pointer_animation<E, Q, F>(f: F, duration: Duration) -> impl ObserverSystem<Pointer<E>, ()>
where
    E: Debug + Clone + Reflect,
    Q: QueryData + Send + Sync + 'static,
    for<'a, 'b> Q::Item<'a, 'b>: QueryData + Send + Sync,
    F: IntoTransformationAnimation<Q>,
{
    IntoSystem::into_system(
        move |event: On<Pointer<E>>, q: Query<Q>, mut commands: Commands| {
            let event = event.event();
            if let Ok(origin) = q.get(event.entity) {
                commands.entity(event.entity).insert(Animation {
                    f: Box::new(f.as_transform(origin)),
                    timer: Timer::new(duration, TimerMode::Once),
                    _c: PhantomData,
                });
            }
        },
    )
}

pub fn animate<Q>(
    animations: Query<(Q, &mut Animation<Q>, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
) where
    Q: QueryData + Send + Sync + 'static,
{
    for (c, mut animation, entity) in animations {
        animation.timer.tick(time.delta());
        let fraction = animation.timer.fraction();
        (animation.f)(c, fraction);
        if animation.timer.is_finished() {
            commands.entity(entity).remove::<Animation<Q>>();
        }
    }
}

pub trait AnimationExt {
    fn transition<C, F>(&mut self, f: F, duration_ms: u64) -> &mut Self
    where
        C: Component<Mutability = Mutable>,
        for<'a> F: IntoTransformationAnimation<&'a mut C>;

    fn animate_event<E, Q, F>(&mut self, f: F, duration: Duration) -> &mut Self
    where
        E: Debug + Clone + Reflect,
        Q: QueryData + Send + Sync + 'static,
        for<'a, 'b> Q::Item<'a, 'b>: QueryData + Send + Sync,
        F: IntoTransformationAnimation<Q>;

    fn animate<IN, OUT, Q, F>(&mut self, f: F, duration: Duration) -> &mut Self
    where
        IN: Debug + Clone + Reflect,
        OUT: Debug + Clone + Reflect,
        Q: QueryData + Send + Sync + 'static,
        for<'a, 'b> Q::Item<'a, 'b>: QueryData + Send + Sync,
        for<'a, 'b> <Q::ReadOnly as QueryData>::Item<'a, 'b>: Clone,
        F: IntoTransformationAnimation<Q>;

    fn animate_hover<Q, F>(&mut self, f: F, duration: Duration) -> &mut Self
    where
        Q: QueryData + Send + Sync + 'static,
        for<'a, 'b> Q::Item<'a, 'b>: QueryData + Send + Sync,
        for<'a, 'b> <Q::ReadOnly as QueryData>::Item<'a, 'b>: Clone,
        F: IntoTransformationAnimation<Q>;

    fn animate_press<Q, F>(&mut self, f: F, duration: Duration) -> &mut Self
    where
        Q: QueryData + Send + Sync + 'static,
        for<'a, 'b> Q::Item<'a, 'b>: QueryData + Send + Sync,
        for<'a, 'b> <Q::ReadOnly as QueryData>::Item<'a, 'b>: Clone,
        F: IntoTransformationAnimation<Q>;

    fn animate_drag<Q, F>(&mut self, f: F, duration: Duration) -> &mut Self
    where
        Q: QueryData + Send + Sync + 'static,
        for<'a, 'b> Q::Item<'a, 'b>: QueryData + Send + Sync,
        for<'a, 'b> <Q::ReadOnly as QueryData>::Item<'a, 'b>: Clone,
        F: IntoTransformationAnimation<Q>;
}

impl AnimationExt for EntityCommands<'_> {
    fn transition<C, F>(&mut self, f: F, duration_ms: u64) -> &mut Self
    where
        C: Component<Mutability = Mutable>,
        for<'a> F: IntoTransformationAnimation<&'a mut C>,
    {
        self.queue_silenced(move |mut entity_world_mut: EntityWorldMut<'_>| {
            if let Some(origin) = entity_world_mut.get() {
                entity_world_mut.insert(Animation::<&mut C>::new(f, duration_ms, origin));
            }
        })
    }

    fn animate_event<E, Q, F>(&mut self, f: F, duration: Duration) -> &mut Self
    where
        E: Debug + Clone + Reflect,
        Q: QueryData + Send + Sync + 'static,
        for<'a, 'b> Q::Item<'a, 'b>: QueryData + Send + Sync,
        F: IntoTransformationAnimation<Q>,
    {
        self.observe(pointer_animation::<E, _, _>(f, duration))
    }

    fn animate<IN, OUT, Q, F>(&mut self, f: F, duration: Duration) -> &mut Self
    where
        IN: Debug + Clone + Reflect,
        OUT: Debug + Clone + Reflect,
        Q: QueryData + Send + Sync + 'static,
        for<'a, 'b> Q::Item<'a, 'b>: QueryData + Send + Sync,
        for<'a, 'b> <Q::ReadOnly as QueryData>::Item<'a, 'b>: Clone,
        F: IntoTransformationAnimation<Q>,
    {
        self.observe(
            move |event: On<Pointer<IN>>, q: Query<Q>, mut commands: Commands| {
                let event = event.event();
                if let Ok(origin) = q.get(event.entity) {
                    let f_reverse = f.reverse(origin.clone());
                    commands
                        .entity(event.entity)
                        .insert(Animation {
                            f: Box::new(f.as_transform(origin)),
                            timer: Timer::new(duration, TimerMode::Once),
                            _c: PhantomData,
                        })
                        .animate_event::<OUT, _, _>(f_reverse, duration);
                }
            },
        )
    }

    fn animate_hover<Q, F>(&mut self, f: F, duration: Duration) -> &mut Self
    where
        Q: QueryData + Send + Sync + 'static,
        for<'a, 'b> Q::Item<'a, 'b>: QueryData + Send + Sync,
        for<'a, 'b> <Q::ReadOnly as QueryData>::Item<'a, 'b>: Clone,
        F: IntoTransformationAnimation<Q>,
    {
        self.animate::<Over, Out, _, _>(f, duration)
    }

    fn animate_press<Q, F>(&mut self, f: F, duration: Duration) -> &mut Self
    where
        Q: QueryData + Send + Sync + 'static,
        for<'a, 'b> Q::Item<'a, 'b>: QueryData + Send + Sync,
        for<'a, 'b> <Q::ReadOnly as QueryData>::Item<'a, 'b>: Clone,
        F: IntoTransformationAnimation<Q>,
    {
        self.animate::<Press, Release, _, _>(f, duration)
    }

    fn animate_drag<Q, F>(&mut self, f: F, duration: Duration) -> &mut Self
    where
        Q: QueryData + Send + Sync + 'static,
        for<'a, 'b> Q::Item<'a, 'b>: QueryData + Send + Sync,
        for<'a, 'b> <Q::ReadOnly as QueryData>::Item<'a, 'b>: Clone,
        F: IntoTransformationAnimation<Q>,
    {
        self.animate::<DragStart, DragEnd, _, _>(f, duration)
    }
}

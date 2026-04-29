use super::IntoTransformationAnimation;
use crate::prelude::*;
use bevy::ecs::query::QueryData;

impl IntoTransformationAnimation<&mut BackgroundColor> for BackgroundColor {
    fn as_transform<'a, 'b>(
        &self,
        origin: <<&'a mut BackgroundColor as QueryData>::ReadOnly as QueryData>::Item<'a, 'b>,
    ) -> impl FnMut(<&mut BackgroundColor as QueryData>::Item<'_, '_>, f32) + Send + Sync + 'static
    {
        let origin = origin.0.to_linear();
        let div = self.0.to_linear() - origin;
        move |mut color, progress| *color = (origin + div * progress).into()
    }
    fn reverse(
        &self,
        origin: <<&mut BackgroundColor as QueryData>::ReadOnly as QueryData>::Item<'_, '_>,
    ) -> Self {
        *origin
    }
}

impl IntoTransformationAnimation<&mut BorderColor> for Color {
    fn as_transform<'a, 'b>(
        &self,
        origin: <<&'a mut BorderColor as QueryData>::ReadOnly as QueryData>::Item<'a, 'b>,
    ) -> impl FnMut(<&mut BorderColor as QueryData>::Item<'_, '_>, f32) + Send + Sync + 'static
    {
        let origin = origin.top.to_linear();
        let div = self.to_linear() - origin;
        move |mut bc, progress| *bc = BorderColor::all(origin + div * progress)
    }
    fn reverse(
        &self,
        origin: <<&mut BorderColor as QueryData>::ReadOnly as QueryData>::Item<'_, '_>,
    ) -> Self {
        origin.top
    }
}

impl IntoTransformationAnimation<&mut Transform> for Quat {
    fn as_transform<'a, 'b>(
        &self,
        origin: <<&'a mut Transform as QueryData>::ReadOnly as QueryData>::Item<'a, 'b>,
    ) -> impl FnMut(<&mut Transform as QueryData>::Item<'_, '_>, f32) + Send + Sync + 'static {
        let origin = origin.rotation;
        let goal = *self;
        move |mut pos, progress| pos.rotation = origin.slerp(goal, progress)
    }
    fn reverse(
        &self,
        origin: <<&mut Transform as QueryData>::ReadOnly as QueryData>::Item<'_, '_>,
    ) -> Self {
        origin.rotation
    }
}

impl<B, F, Q, T> IntoTransformationAnimation<Q> for (B, T)
where
    B: Fn((<<Q as QueryData>::ReadOnly as QueryData>::Item<'_, '_>, &T)) -> F
        + Clone
        + Send
        + Sync
        + 'static,
    F: FnMut(<Q as QueryData>::Item<'_, '_>, f32) + Send + Sync + 'static,
    Q: QueryData,
    T: Clone + Send + Sync + 'static,
    for<'a, 'b> <<Q as QueryData>::ReadOnly as QueryData>::Item<'a, 'b>: Deref<Target = T>,
{
    fn as_transform<'a, 'b>(
        &self,
        origin: <<Q as QueryData>::ReadOnly as QueryData>::Item<'a, 'b>,
    ) -> impl FnMut(<Q as QueryData>::Item<'_, '_>, f32) + Send + Sync + 'static {
        self.0((origin, &self.1))
    }
    fn reverse(&self, origin: <<Q as QueryData>::ReadOnly as QueryData>::Item<'_, '_>) -> Self {
        (self.0.clone(), origin.clone())
    }
}

impl<B, F, Q, T, G> IntoTransformationAnimation<Q> for (B, T, G)
where
    B: Fn((<<Q as QueryData>::ReadOnly as QueryData>::Item<'_, '_>, &T)) -> F
        + Clone
        + Send
        + Sync
        + 'static,
    F: FnMut(<Q as QueryData>::Item<'_, '_>, f32) + Send + Sync + 'static,
    Q: QueryData,
    T: Send + Sync + 'static,
    G: Fn(<<Q as QueryData>::ReadOnly as QueryData>::Item<'_, '_>) -> T
        + Clone
        + Send
        + Sync
        + 'static,
{
    fn as_transform<'a, 'b>(
        &self,
        origin: <<Q as QueryData>::ReadOnly as QueryData>::Item<'a, 'b>,
    ) -> impl FnMut(<Q as QueryData>::Item<'_, '_>, f32) + Send + Sync + 'static {
        self.0((origin, &self.1))
    }
    fn reverse(&self, origin: <<Q as QueryData>::ReadOnly as QueryData>::Item<'_, '_>) -> Self {
        (self.0.clone(), self.2(origin), self.2.clone())
    }
}

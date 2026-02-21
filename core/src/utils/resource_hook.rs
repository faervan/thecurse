use crate::prelude::*;

pub trait ResourceTransformHook {
    /// When the input resource (`IR`) is added, remove it and create the output resource (`OR`)
    /// from its value, inserting it into the world.
    /// Basically, replace `IR` with `OR` based on the `transform` function.
    fn transform_resource_on_add<IR, OR, F>(&mut self, transform: F) -> &mut Self
    where
        IR: Resource,
        OR: Resource,
        F: Fn(&mut World, IR) -> OR + Send + Sync + 'static;
}

impl ResourceTransformHook for App {
    fn transform_resource_on_add<IR, OR, F>(&mut self, transform: F) -> &mut Self
    where
        IR: Resource,
        OR: Resource,
        F: Fn(&mut World, IR) -> OR + Send + Sync + 'static,
    {
        self.add_systems(
            Update,
            (move |world: &mut World| {
                let ir = world.remove_resource::<IR>().unwrap();
                let or = transform(world, ir);
                world.insert_resource(or);
            })
            .run_if(resource_added::<IR>),
        );
        self
    }
}

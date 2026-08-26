use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<LoadingAssetHandles>();
    app.add_systems(Update, load_assets);
}

pub fn all_assets_loaded(loader: Res<LoadingAssetHandles>) -> bool {
    loader.loading.is_empty()
}

type InsertAssetResource = Box<dyn FnOnce(&mut World, UntypedHandle) + Send + Sync>;

#[derive(Resource, Default)]
pub struct LoadingAssetHandles {
    loading: Vec<(UntypedHandle, InsertAssetResource)>,
}

pub trait AssetResourceLoader {
    fn load_assets<T>(&mut self) -> &mut Self
    where
        T: Resource + Asset + FromWorld;

    /// Load the given type [`T`] as an [`Asset`], initializing it with its [`FromWorld`] impl. Once
    /// fully loaded, apply the provided transform hook and insert the produced [`Resource`] into
    /// the world.
    fn load_assets_with<T, R, F>(&mut self, on_insert_transform_hook: F) -> &mut Self
    where
        T: Asset + FromWorld,
        R: Resource,
        F: FnOnce(T, &mut World) -> R + Send + Sync + 'static;
}

impl AssetResourceLoader for App {
    fn load_assets<T>(&mut self) -> &mut Self
    where
        T: Resource + Asset + FromWorld,
    {
        self.load_assets_with(|t: T, _world| t)
    }

    fn load_assets_with<T, R, F>(&mut self, on_insert_transform_hook: F) -> &mut Self
    where
        T: Asset + FromWorld,
        R: Resource,
        F: FnOnce(T, &mut World) -> R + Send + Sync + 'static,
    {
        self.init_asset::<T>();

        let world = self.world_mut();
        let t = T::from_world(world);
        let handle = world.resource::<AssetServer>().add(t);
        let mut loading_handles = world.resource_mut::<LoadingAssetHandles>();
        loading_handles.loading.push((
            handle.untyped(),
            Box::new(move |world, handle| {
                let mut assets = world.resource_mut::<Assets<T>>();
                if let Some(value) = assets.remove(handle.id().typed()) {
                    let value = on_insert_transform_hook(value, world);
                    world.insert_resource(value);
                }
            }),
        ));
        self
    }
}

fn load_assets(world: &mut World) {
    world.resource_scope(|world, mut loading_handles: Mut<LoadingAssetHandles>| {
        world.resource_scope(|world, asset_server: Mut<AssetServer>| {
            for i in (0..loading_handles.loading.len()).rev() {
                if asset_server.is_loaded_with_dependencies(&loading_handles.loading[i].0) {
                    let (handle, insert_fn) = loading_handles.loading.swap_remove(i);
                    insert_fn(world, handle);
                }
            }
        })
    })
}

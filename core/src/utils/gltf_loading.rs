use bevy::asset::ReflectAsset;

use crate::prelude::*;

#[derive(Resource, Asset, Reflect)]
#[reflect(Resource, Asset)]
pub struct GltfLoadingHandle<T>
where
    T: GltfAssetPath + TypePath + Send + Sync,
{
    #[dependency]
    pub handle: Handle<Gltf>,
    _phantom: PhantomData<T>,
}

pub trait GltfAssetPath {
    const PATH: &'static str;
}

impl<T> FromWorld for GltfLoadingHandle<T>
where
    T: GltfAssetPath + TypePath + Send + Sync,
{
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        Self {
            handle: asset_server.load(T::PATH),
            _phantom: PhantomData,
        }
    }
}

impl<T> GltfLoadingHandle<T>
where
    T: GltfAssetPath + TypePath + Send + Sync,
{
    pub fn get_gltf<'a>(&self, world: &'a World) -> &'a Gltf {
        let gltfs = world.resource::<Assets<Gltf>>();
        gltfs.get(&self.handle).unwrap()
    }
}

type GetterFn<'a> = dyn FnMut(&str) -> Result<(), GltfAnimationExtractionError> + 'a;

pub trait GltfAnimationExtractionExt {
    fn get_animations<F>(
        &self,
        extraction_fn: F,
    ) -> Result<(AnimationGraph, HashMap<String, AnimationNodeIndex>), GltfAnimationExtractionError>
    where
        F: FnOnce(&mut GetterFn<'_>) -> Result<(), GltfAnimationExtractionError>;
}

#[derive(Error, Debug)]
pub enum GltfAnimationExtractionError {
    #[error("No Animation named {0} found in the gltf")]
    AnimationNotFound(String),
}

impl GltfAnimationExtractionExt for Gltf {
    fn get_animations<F>(
        &self,
        extraction_fn: F,
    ) -> Result<(AnimationGraph, HashMap<String, AnimationNodeIndex>), GltfAnimationExtractionError>
    where
        F: FnOnce(&mut GetterFn<'_>) -> Result<(), GltfAnimationExtractionError>,
    {
        let mut named_clips = HashMap::new();
        let mut getter = |name: &str| match self.named_animations.get(name) {
            Some(clip) => {
                named_clips.insert(name.to_string(), clip.clone());
                Ok(())
            }
            None => Err(GltfAnimationExtractionError::AnimationNotFound(
                name.to_string(),
            )),
        };
        extraction_fn(&mut getter)?;
        let (names, clips): (Vec<_>, Vec<_>) = named_clips.into_iter().unzip();
        let (graph, indices) = AnimationGraph::from_clips(clips);
        Ok((graph, names.into_iter().zip(indices).collect()))
    }
}

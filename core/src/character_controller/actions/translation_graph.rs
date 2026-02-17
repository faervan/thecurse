use crate::prelude::*;

#[derive(Default, Debug)]
pub enum TranslationGraph {
    #[default]
    None,
    Constant(Vec3),
}

impl TranslationGraph {
    /// `delta` is the change since last frame
    pub fn run_step(&self, delta: f32) -> Vec3 {
        match self {
            Self::None => Vec3::ZERO,
            Self::Constant(t) => t * delta,
        }
    }
}

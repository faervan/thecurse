use crate::prelude::*;

#[derive(Reflect, Default, Debug)]
#[reflect(Default)]
pub enum TranslationReference {
    #[default]
    /// Translation is applied relative to its own rotation.
    Local,
    /// Translation is applied in world space.
    World,
    /// Translation is applied relative to the rotation of [Entity]
    Entity(Entity),
}

#[derive(Reflect, Default, Debug)]
#[reflect(Default)]
/// Mask the [`TranslationReference`], so that only certain parts of the forward vector of the
/// reference are considered.
pub enum TranslationReferenceMask {
    #[default]
    /// Take all parts of the forward vector
    All,
    /// Exclude the y part, useful when e.g. making sure that movement translation is always
    /// relative to the XZ plane.
    XZ,
    /// Exclude the z part
    XY,
    /// Exclude the x part
    ZY,
}

impl TranslationReferenceMask {
    pub fn apply_to(&self, vector: &mut Vec3) {
        match self {
            Self::All => (),
            Self::XZ => vector.y = 0.,
            Self::XY => vector.z = 0.,
            Self::ZY => vector.x = 0.,
        }
    }
}

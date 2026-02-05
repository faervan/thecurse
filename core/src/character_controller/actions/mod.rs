use crate::prelude::*;

pub struct CharacterAction {
    translation: TranslationGraph,
    animation: (),
    duration: (),
}

enum TranslationGraph {
    Constant(Vec3),
}

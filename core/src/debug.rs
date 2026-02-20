use std::{fmt::Debug, hash::Hash};

use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.insert_state(DebugElementActive::<WorldInspectorPlugin>::default());
    app.add_plugins((
        EguiPlugin::default(),
        WorldInspectorPlugin::default()
            .run_if(in_state(DebugElementActive::<WorldInspectorPlugin>::ACTIVE)),
    ));

    app.add_systems(Update, toggle_debug_element::<3, WorldInspectorPlugin>);
}

const fn key_map(id: usize) -> KeyCode {
    match id {
        3 => KeyCode::F3,
        _ => panic!(),
    }
}

fn toggle_debug_element<const KEY: usize, T: Send + Sync + 'static>(
    input: Res<ButtonInput<KeyCode>>,
    state: Res<State<DebugElementActive<T>>>,
    mut next_state: ResMut<NextState<DebugElementActive<T>>>,
) {
    if input.just_pressed(key_map(KEY)) {
        next_state.set(DebugElementActive::new(!state.get().active));
    }
}

#[derive(States, Default)]
struct DebugElementActive<T: Send + Sync + 'static> {
    active: bool,
    _phantom: PhantomData<T>,
}

impl<T: Send + Sync + 'static> DebugElementActive<T> {
    const ACTIVE: Self = Self {
        active: true,
        _phantom: PhantomData,
    };

    const fn new(active: bool) -> Self {
        Self {
            active,
            _phantom: PhantomData,
        }
    }
}

impl<T: Send + Sync + 'static> Clone for DebugElementActive<T> {
    fn clone(&self) -> Self {
        Self {
            active: self.active,
            _phantom: PhantomData,
        }
    }
}

impl<T: Send + Sync + 'static> Debug for DebugElementActive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DebugElementActive {{ active: {} }}", self.active)
    }
}

impl<T: Send + Sync + 'static> PartialEq for DebugElementActive<T> {
    fn eq(&self, other: &Self) -> bool {
        self.active == other.active
    }
}

impl<T: Send + Sync + 'static> Eq for DebugElementActive<T> {}

impl<T: Send + Sync + 'static> Hash for DebugElementActive<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.active.hash(state);
    }
}

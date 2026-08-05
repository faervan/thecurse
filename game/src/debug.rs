use std::{fmt::Debug, hash::Hash};

use bevy::dev_tools::picking_debug::{DebugPickingMode, DebugPickingPlugin};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    WorldInspectorPlugin::debug_plugin(app);
    CustomDiagnosticPlugin::debug_plugin(app);
    DebugPickingPlugin::debug_plugin(app);
    PhysicsDebugPlugin::debug_plugin(app);
    NavMeshDebug::debug_plugin(app);
}

trait DebugElement: Sized + Send + Sync + 'static {
    const KEY: KeyCode;

    fn debug_plugin(app: &mut App) -> &mut App {
        app.insert_state(DebugElementActive::<Self>::default());
        app.add_systems(Update, Self::toggle_on_key_press);
        Self::plugin(app);
        if let Some(on_enable) = Self::on_enable() {
            app.add_systems(OnEnter(DebugElementActive::<Self>::ACTIVE), on_enable);
        }
        if let Some(on_disable) = Self::on_disable() {
            app.add_systems(OnExit(DebugElementActive::<Self>::ACTIVE), on_disable);
        }
        app
    }

    #[allow(unused_variables)]
    fn plugin(app: &mut App) {}

    fn on_enable() -> Option<impl System<In = (), Out = ()>> {
        // Necessary for compiler type inference
        if false {
            return Some(IntoSystem::into_system(|| {}));
        }
        None
    }

    fn on_disable() -> Option<impl System<In = (), Out = ()>> {
        // Necessary for compiler type inference
        if false {
            return Some(IntoSystem::into_system(|| {}));
        }
        None
    }

    fn toggle_on_key_press(
        input: Res<ButtonInput<KeyCode>>,
        state: Res<State<DebugElementActive<Self>>>,
        mut next_state: ResMut<NextState<DebugElementActive<Self>>>,
    ) {
        if input.just_pressed(Self::KEY) {
            next_state.set(DebugElementActive::new(!state.get().active));
        }
    }
}

macro_rules! on_enable {
    ($system:expr) => {
        fn on_enable() -> Option<impl System<In = (), Out = ()>> {
            Some(IntoSystem::into_system($system))
        }
    };
}

macro_rules! on_disable {
    ($system:expr) => {
        fn on_disable() -> Option<impl System<In = (), Out = ()>> {
            Some(IntoSystem::into_system($system))
        }
    };
}

impl DebugElement for WorldInspectorPlugin {
    const KEY: KeyCode = KeyCode::F1;

    fn plugin(app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }
        app.add_plugins(Self::default().run_if(in_state(DebugElementActive::<Self>::ACTIVE)));
    }
}

#[derive(Resource, Default)]
struct CustomDiagnosticPlugin {
    updates: usize,
    last_updates: usize,
    updates_timer: Timer,
    fixed_updates: usize,
    last_fixed_updates: usize,
    fixed_updates_timer: Timer,
}

#[derive(Component)]
struct CustomDiagnosticUi;

#[derive(Component)]
struct CustomDiagnosticUiText;

impl DebugElement for CustomDiagnosticPlugin {
    const KEY: KeyCode = KeyCode::F2;

    fn plugin(app: &mut App) {
        app.insert_resource(Self {
            updates_timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
            fixed_updates_timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
            ..Default::default()
        });

        app.add_systems(Update, update);
        app.add_systems(FixedUpdate, fixed_update);

        app.add_systems(
            Update,
            update_text.run_if(in_state(DebugElementActive::<Self>::ACTIVE)),
        );
    }

    on_enable!(|mut commands: Commands| {
        commands
            .spawn((
                CustomDiagnosticUi,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    right: Val::Px(10.0),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn((Text::new("Diagnostics"), CustomDiagnosticUiText));
            });
    });

    on_disable!(
        |mut commands: Commands, query: Query<Entity, With<CustomDiagnosticUi>>| {
            for entity in query {
                commands.entity(entity).despawn();
            }
        }
    );
}

fn update_text(
    diagnostic: Res<CustomDiagnosticPlugin>,
    query: Query<&mut Text, With<CustomDiagnosticUiText>>,
) {
    for mut text in query {
        text.0 = format!(
            "Updates: {}Hz\nFixed Updates: {}Hz",
            diagnostic.last_updates, diagnostic.last_fixed_updates
        );
    }
}

fn update(time: Res<Time>, mut diagnostic: ResMut<CustomDiagnosticPlugin>) {
    diagnostic.updates += 1;
    diagnostic.updates_timer.tick(time.delta());
    if diagnostic.updates_timer.just_finished() {
        diagnostic.last_updates = diagnostic.updates;
        diagnostic.updates = 0;
    }
}

fn fixed_update(time: Res<Time>, mut diagnostic: ResMut<CustomDiagnosticPlugin>) {
    diagnostic.fixed_updates += 1;
    diagnostic.fixed_updates_timer.tick(time.delta());
    if diagnostic.fixed_updates_timer.just_finished() {
        diagnostic.last_fixed_updates = diagnostic.fixed_updates;
        diagnostic.fixed_updates = 0;
    }
}

impl DebugElement for DebugPickingPlugin {
    const KEY: KeyCode = KeyCode::F3;

    fn plugin(app: &mut App) {
        app.add_plugins(DebugPickingPlugin);
        app.insert_resource(DebugPickingMode::Disabled);
    }

    on_enable!(|mut mode: ResMut<DebugPickingMode>| {
        *mode = DebugPickingMode::Normal;
    });

    on_disable!(|mut mode: ResMut<DebugPickingMode>| {
        *mode = DebugPickingMode::Disabled;
    });
}

impl DebugElement for PhysicsDebugPlugin {
    const KEY: KeyCode = KeyCode::F4;

    fn plugin(app: &mut App) {
        app.add_plugins(Self);
        app.insert_gizmo_config(
            PhysicsGizmos::default(),
            GizmoConfig {
                enabled: false,
                ..Default::default()
            },
        );

        app.add_systems(
            Update,
            (|mut gizmos: Gizmos, cursor_target: Res<CursorTargetPosition>| {
                if let Some(position) = **cursor_target {
                    gizmos.sphere(position, 0.2, Color::srgb(0.5, 0.5, 0.1));
                }
            })
            .run_if(in_state(DebugElementActive::<Self>::ACTIVE)),
        );
    }

    on_enable!(|mut configs: ResMut<GizmoConfigStore>| {
        if let Some((gizmo_config, _physics_gizmo)) =
            configs.get_config_mut_dyn(&std::any::TypeId::of::<PhysicsGizmos>())
        {
            gizmo_config.enabled = true;
        }
    });

    on_disable!(|mut configs: ResMut<GizmoConfigStore>| {
        if let Some((gizmo_config, _physics_gizmo)) =
            configs.get_config_mut_dyn(&std::any::TypeId::of::<PhysicsGizmos>())
        {
            gizmo_config.enabled = false;
        }
    });
}

impl DebugElement for NavMeshDebug {
    const KEY: KeyCode = KeyCode::F5;

    fn plugin(app: &mut App) {
        app.add_systems(
            Update,
            (|mut gizmos: Gizmos, query: Query<(&CreatureNavmeshPath, &Transform)>| {
                for (path, transform) in query {
                    let mut last = transform.translation;

                    for next in path.path.iter().rev() {
                        let next = vec3(next.x, 0., next.y);
                        gizmos.line(last, next, bevy::color::palettes::css::ORANGE);
                        last = next;
                    }
                }
            })
            .run_if(in_state(DebugElementActive::<Self>::ACTIVE)),
        );
    }

    on_enable!(
        |mut commands: Commands, navmeshes: Query<Entity, With<ManagedNavMesh>>| {
            for entity in navmeshes {
                commands
                    .entity(entity)
                    .insert(NavMeshDebug(bevy::color::palettes::css::RED.into()));
            }
        }
    );

    on_disable!(
        |mut commands: Commands, navmeshes: Query<Entity, With<ManagedNavMesh>>| {
            for entity in navmeshes {
                commands.entity(entity).remove::<NavMeshDebug>();
            }
        }
    );
}

#[derive(States)]
struct DebugElementActive<T: DebugElement> {
    active: bool,
    _phantom: PhantomData<T>,
}

impl<T: DebugElement> DebugElementActive<T> {
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

impl<T: DebugElement> Default for DebugElementActive<T> {
    fn default() -> Self {
        Self {
            active: false,
            _phantom: PhantomData,
        }
    }
}

impl<T: DebugElement> Clone for DebugElementActive<T> {
    fn clone(&self) -> Self {
        Self {
            active: self.active,
            _phantom: PhantomData,
        }
    }
}

impl<T: DebugElement> Debug for DebugElementActive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DebugElementActive {{ active: {} }}", self.active)
    }
}

impl<T: DebugElement> PartialEq for DebugElementActive<T> {
    fn eq(&self, other: &Self) -> bool {
        self.active == other.active
    }
}

impl<T: DebugElement> Eq for DebugElementActive<T> {}

impl<T: DebugElement> Hash for DebugElementActive<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.active.hash(state);
    }
}

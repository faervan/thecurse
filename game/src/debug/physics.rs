use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<PhysicsDebugEnabled>();

    app.add_plugins(PhysicsDebugPlugin);
    app.insert_gizmo_config(
        PhysicsGizmos::default(),
        GizmoConfig {
            enabled: false,
            ..Default::default()
        },
    );

    app.add_systems(
        Update,
        (|mut next_state: ResMut<NextState<PhysicsDebugEnabled>>,
          state: Res<State<PhysicsDebugEnabled>>| {
            next_state.set(PhysicsDebugEnabled(!state.get().0));
        })
        .run_if(input_just_pressed(KeyCode::F4)),
    );

    app.add_systems(
        OnEnter(PhysicsDebugEnabled(true)),
        |mut configs: ResMut<GizmoConfigStore>| {
            if let Some((gizmo_config, _physics_gizmo)) =
                configs.get_config_mut_dyn(&std::any::TypeId::of::<PhysicsGizmos>())
            {
                gizmo_config.enabled = true;
            }
        },
    );

    app.add_systems(
        OnExit(PhysicsDebugEnabled(true)),
        |mut configs: ResMut<GizmoConfigStore>| {
            if let Some((gizmo_config, _physics_gizmo)) =
                configs.get_config_mut_dyn(&std::any::TypeId::of::<PhysicsGizmos>())
            {
                gizmo_config.enabled = false;
            }
        },
    );

    app.add_systems(
        Update,
        (|mut gizmos: Gizmos, cursor_target: Res<CursorTargetPosition>| {
            if let Some(position) = **cursor_target {
                gizmos.sphere(position, 0.2, Color::srgb(0.5, 0.5, 0.1));
            }
        })
        .run_if(in_state(PhysicsDebugEnabled(true))),
    );
}

#[derive(States, Hash, Clone, Copy, PartialEq, Eq, Default, Debug)]
struct PhysicsDebugEnabled(bool);

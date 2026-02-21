use thecurse_core::{
    ActionHook, ActionHookAction, ActionHookCondition, ActionIndex, CameraController,
    CharacterAction, CharacterController, TranslationGraph, TranslationReference,
    TranslationReferenceMask,
};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::Game),
        spawn_player.after(thecurse_core::spawn_camera),
    );
    app.add_systems(Update, player_input);
}

#[derive(Component)]
struct Player {
    forward: ActionIndex,
    backward: ActionIndex,
    left: ActionIndex,
    right: ActionIndex,
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera: Single<Entity, With<CameraController>>,
) {
    let speed = 50.;

    let action = |vec: Vec3| -> CharacterAction {
        let mut action = CharacterAction::new(
            TranslationGraph::Constant(vec),
            Default::default(),
            Duration::from_millis(100),
        )
        .set_translation_multiplier(speed)
        .set_repeat();
        action.transition_reference = TranslationReference::Entity(*camera);
        action.transition_reference_mask = TranslationReferenceMask::XZ;
        action
    };

    let actions = vec![
        // 0: Forward
        action(Vec3::NEG_Z),
        // 1: Backward
        action(Vec3::Z),
        // 2: Left
        action(Vec3::NEG_X),
        // 3: Right
        action(Vec3::X),
        // 4: Forward left
        action((Vec3::NEG_Z + Vec3::NEG_X).normalize()),
        // 5: Forward right
        action((Vec3::NEG_Z + Vec3::X).normalize()),
        // 6: Backward left
        action((Vec3::Z + Vec3::NEG_X).normalize()),
        // 7: Backward right
        action((Vec3::Z + Vec3::X).normalize()),
    ];
    let (mut controller, _) = CharacterController::from_actions(actions);

    let mut diagonal_hooks = |a: ActionIndex, b: ActionIndex, diagonal: ActionIndex| {
        controller.add_action_start_hook(
            a,
            ActionHook {
                condition: ActionHookCondition::IsRunning(b),
                actions: vec![
                    ActionHookAction::Remove(a),
                    ActionHookAction::Remove(b),
                    ActionHookAction::Insert(diagonal),
                ],
            },
        );
        controller.add_action_start_hook(
            b,
            ActionHook {
                condition: ActionHookCondition::IsRunning(a),
                actions: vec![
                    ActionHookAction::Remove(a),
                    ActionHookAction::Remove(b),
                    ActionHookAction::Insert(diagonal),
                ],
            },
        );
        controller.add_action_end_hook(
            a,
            ActionHook {
                condition: ActionHookCondition::IsRunning(diagonal),
                actions: vec![
                    ActionHookAction::Remove(diagonal),
                    ActionHookAction::Insert(b),
                ],
            },
        );
        controller.add_action_end_hook(
            b,
            ActionHook {
                condition: ActionHookCondition::IsRunning(diagonal),
                actions: vec![
                    ActionHookAction::Remove(diagonal),
                    ActionHookAction::Insert(a),
                ],
            },
        );
    };

    // Forward left
    diagonal_hooks(0, 2, 4);
    // Forward right
    diagonal_hooks(0, 3, 5);
    // Backward left
    diagonal_hooks(1, 2, 6);
    // Backward right
    diagonal_hooks(1, 3, 7);

    let id = commands
        .spawn((
            Name::new("Player"),
            Mesh3d(meshes.add(Capsule3d::new(0.3, 2.))),
            MeshMaterial3d(materials.add(StandardMaterial::from_color(Color::BLACK))),
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            Collider::cuboid(0.5, 2., 0.2),
            GravityScale(0.),
            controller,
            Player {
                forward: 0,
                backward: 1,
                left: 2,
                right: 3,
            },
        ))
        .id();

    commands.entity(*camera).insert(ChildOf(id));
}

fn player_input(
    input: Res<ButtonInput<KeyCode>>,
    mut player: Single<(&mut CharacterController, &Player)>,
) {
    if input.just_pressed(KeyCode::KeyW) {
        let forward = player.1.forward;
        player.0.start_action(forward);
    }
    if input.just_released(KeyCode::KeyW) {
        let forward = player.1.forward;
        player.0.end_action(forward);
    }

    if input.just_pressed(KeyCode::KeyS) {
        let backward = player.1.backward;
        player.0.start_action(backward);
    }
    if input.just_released(KeyCode::KeyS) {
        let backward = player.1.backward;
        player.0.end_action(backward);
    }

    if input.just_pressed(KeyCode::KeyA) {
        let left = player.1.left;
        player.0.start_action(left);
    }
    if input.just_released(KeyCode::KeyA) {
        let left = player.1.left;
        player.0.end_action(left);
    }

    if input.just_pressed(KeyCode::KeyD) {
        let right = player.1.right;
        player.0.start_action(right);
    }
    if input.just_released(KeyCode::KeyD) {
        let right = player.1.right;
        player.0.end_action(right);
    }
}

use thecurse_core::{
    ActionIndex, CameraControllerAnchor, CharacterAction, CharacterController, TranslationGraph,
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
    ankor: Single<Entity, With<CameraControllerAnchor>>,
) {
    let speed = 50.;
    let actions = vec![
        // Forward
        CharacterAction::new(
            TranslationGraph::Constant(Vec3::NEG_Z),
            Default::default(),
            Duration::from_millis(100),
        )
        .set_translation_multiplier(speed)
        .set_repeat(),
        // Backward
        CharacterAction::new(
            TranslationGraph::Constant(Vec3::Z),
            Default::default(),
            Duration::from_millis(100),
        )
        .set_translation_multiplier(speed)
        .set_repeat(),
        // Left
        CharacterAction::new(
            TranslationGraph::Constant(Vec3::NEG_X),
            Default::default(),
            Duration::from_millis(100),
        )
        .set_translation_multiplier(speed)
        .set_repeat(),
        // Right
        CharacterAction::new(
            TranslationGraph::Constant(Vec3::X),
            Default::default(),
            Duration::from_millis(100),
        )
        .set_translation_multiplier(speed)
        .set_repeat(),
    ];
    let (controller, _) = CharacterController::from_actions(actions);
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

    commands.entity(*ankor).insert(ChildOf(id));
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

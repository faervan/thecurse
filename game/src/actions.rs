use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_action::<JumpAction>();
    app.play_action_while_pressed::<JumpAction, _>(KeyCode::Space);

    app.register_action::<MovementAction>();
    app.add_systems(Update, set_movement);
}

#[derive(Bundle, Default)]
pub struct PlayerController {
    jump: ActionPlayer<JumpAction>,
    movement: ActionPlayer<MovementAction>,
}

#[derive(Default)]
struct JumpAction;

impl Action for JumpAction {
    const CANCELS: &[ActionId] = &[];

    type ActionStartQuery = (&'static Name, &'static mut LinearVelocity);
    type ActionStartParam<'w, 's> = Commands<'w, 's>;
    fn on_action_start<'w, 's>(
        &mut self,
        (name, mut velocity): <Self::ActionStartQuery as QueryData>::Item<'_, '_>,
        _commands: StaticSystemParam<Self::ActionStartParam<'w, 's>>,
    ) {
        debug!("{} jumps", name);
        if velocity.y.abs() < 1. {
            velocity.y = 40.;
        }
    }

    type ActionStopQuery = ();
    type ActionStopParam<'w, 's> = ();

    type ActionChangeQuery = ();
    type ActionChangeParam<'w, 's> = ();
}

#[derive(Default)]
struct MovementAction;

impl Action for MovementAction {
    const CANCELS: &[ActionId] = &[];

    type ActionStartQuery = ();
    type ActionStartParam<'w, 's> = ();

    type ActionStopQuery = ();
    type ActionStopParam<'w, 's> = ();

    type ActionChangeQuery = ();
    type ActionChangeParam<'w, 's> = ();
}

fn set_movement(
    input: Res<ButtonInput<KeyCode>>,
    query: Query<(&mut ActionPlayer<MovementAction>, &mut LinearVelocity)>,
    camera: Single<&Transform, With<Camera3d>>,
) {
    for (mut player, mut velocity) in query {
        let any_input =
            input.any_pressed([KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD]);
        if !player.is_active() && !any_input {
            return;
        } else if !any_input {
            player.stop();
            velocity.x = 0.;
            velocity.z = 0.;
            return;
        } else if !player.is_active() {
            player.start();
        }

        let mut dir = Vec3::ZERO;
        if input.pressed(KeyCode::KeyW) {
            dir.z -= 1.;
        }
        if input.pressed(KeyCode::KeyA) {
            dir.x -= 1.;
        }
        if input.pressed(KeyCode::KeyS) {
            dir.z += 1.;
        }
        if input.pressed(KeyCode::KeyD) {
            dir.x += 1.;
        }

        dir = camera.rotation * dir.normalize_or_zero() * 20.;
        velocity.x = dir.x;
        velocity.z = dir.z;
    }
}

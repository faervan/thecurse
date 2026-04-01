use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_action::<JumpAction>();
    app.play_action_while_pressed::<JumpAction, _>(KeyCode::Space);
}

pub struct JumpAction;

impl Action for JumpAction {
    const CANCELS: &[ActionId] = &[];

    type ActionStartParam<'w, 's> = Commands<'w, 's>;
    type ActionStartQuery = (&'static Name, &'static mut LinearVelocity);
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

    type ActionStopParam<'w, 's> = ();
    type ActionStopQuery = ();
    fn on_action_stop<'w, 's>(
        &mut self,
        _query: <Self::ActionStopQuery as QueryData>::Item<'_, '_>,
        _params: StaticSystemParam<Self::ActionStopParam<'w, 's>>,
    ) {
    }
}

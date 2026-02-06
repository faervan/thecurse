use bitflags::bitflags;

use crate::prelude::*;

bitflags! {
    pub struct ActionLayer: u16 {
        /// Action set containing basic movement
        const Movement = 1;
        /// Action set containing dash-like movement
        const Dash = 1 << 1;
        /// Action set containing jump-like movement
        const Jump = 1 << 2;
        /// Action set containing light actions that can be performed simultaneously with other
        /// actions like basic movement.
        const LightAction = 1 << 3;
        /// Action set containing actions that cannot be performed concurrently to other actions.
        const FocusedAction = 1 << 4;

        /// Whether this action can be interrupted by other actions that cannot be run
        /// concurrently.
        const Interruptible = 1 << 5;
        /// Whether this action can be canceled.
        const Cancelable = 1 << 6;
    }
}

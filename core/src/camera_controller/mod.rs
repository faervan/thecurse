use crate::prelude::*;

/// Camera spawning
mod setup;

/// Camera settings
mod settings;
pub use settings::CameraControllerSettings;

/// Camera zoom and rotation
mod movement;

/// Enforce a line of sight from camera to camera anchor
mod line_of_sight;

#[derive(Default)]
/// A third person [`CameraController`] that can be zoomed and orbited around a center point called
/// the [`CameraControllerAnchor`].
pub struct CameraControllerPlugin<STATE> {
    /// Default camera settings. This is a [`Resource`].
    pub settings: CameraControllerSettings,
    state: STATE,
}

impl<STATE> CameraControllerPlugin<STATE> {
    pub fn new(state: STATE) -> Self {
        Self {
            state,
            settings: Default::default(),
        }
    }
}

impl<STATE> Plugin for CameraControllerPlugin<STATE>
where
    STATE: States + Copy,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(self.settings.clone())
            .add_systems(OnEnter(self.state), setup::setup)
            .add_systems(OnExit(self.state), setup::despawn)
            .add_systems(
                Update,
                (
                    movement::zoom,
                    movement::rotate,
                    line_of_sight::enforce
                        .in_set(PhysicsSystems::Last)
                        .after(movement::zoom),
                )
                    .run_if(in_state(self.state)),
            );
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
/// The [`Entity`] with this component should always be a child of a [`CameraControllerAnchor`].
pub struct CameraController {
    /// The wanted distance from the camera origin. The actual distance might be smaller to prevent
    /// glitching into walls.
    distance: f32,
    /// Origin of the camera, meaning the point to which the [`CameraController`] is looking.
    /// Should be [`Vec3::ZERO`] (meaning it will look towards the [`CameraControllerAnchor`]),
    /// but may be offset if the [`CameraControllerAnchor`] is to close to an obstacle.
    origin: Vec3,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CameraControllerAnchor;

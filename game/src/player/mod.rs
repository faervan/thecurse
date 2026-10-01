use crate::prelude::*;

pub mod cursor_target;
mod movement;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((cursor_target::plugin, movement::plugin));
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Name::new("MainCharacter"), PlayerCharacter)]
pub struct MainCharacter {
    last_movement_direction: Vec3,
    action_timer: Timer,
}

impl Default for MainCharacter {
    fn default() -> Self {
        Self {
            last_movement_direction: Vec3::ZERO,
            action_timer: Timer::new(Duration::from_millis(50), TimerMode::Repeating),
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(PlayerCharacter)]
pub struct ScriptedPlayer {
    pub id: ClientId,
    pub authoritative_state: PlayerState,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Player, GameEntity)]
#[component(on_add)]
struct PlayerCharacter;

impl PlayerCharacter {
    fn on_add(mut world: DeferredWorld, hook: HookContext) {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh = meshes.add(Capsule3d::new(0.2, 1.7));

        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.1),
            ..Default::default()
        });

        world
            .commands()
            .entity(hook.entity)
            .insert((Mesh3d(mesh), MeshMaterial3d(material)));
    }
}

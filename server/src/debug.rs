use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(FixedUpdates {
        count: 0,
        timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
    });

    app.add_systems(FixedUpdate, update);
}

#[derive(Resource)]
struct FixedUpdates {
    count: usize,
    timer: Timer,
}

fn update(time: Res<Time>, mut updates: ResMut<FixedUpdates>) {
    updates.count += 1;
    updates.timer.tick(time.delta());
    if updates.timer.just_finished() {
        // debug!("FixedUpdate ran on {}Hz", updates.count);
        updates.count = 0;
    }
}

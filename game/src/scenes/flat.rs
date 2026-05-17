use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    // app.add_systems(OnEnter(GameScene::Flat), spawn);
}

fn spawn(mut commands: Commands) {
    commands.spawn(RasterizedGridObj);
    commands.spawn((PointLightObj, Transform::from_xyz(5., 3., 3.)));
    commands.spawn((RockObj, Transform::from_xyz(10., 2.5, 10.)));
}

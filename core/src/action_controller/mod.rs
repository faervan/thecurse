use std::{
    any::{Any, TypeId},
    hash::Hash,
};

use bevy::ecs::system::SystemParam;

use crate::prelude::*;

fn plugin<A>(app: &mut App)
where
    A: Action,
{
    app.add_message::<ActionStart<A>>();
    app.add_message::<ActionStop<A>>();
    app.add_message::<ActionChange<A>>();

    app.add_systems(
        PostUpdate,
        (
            announce_action_state_changes::<A>,
            apply_action_start::<A>.after(announce_action_state_changes::<A>),
            apply_action_stop::<A>.after(announce_action_state_changes::<A>),
            apply_action_change::<A>.after(announce_action_state_changes::<A>),
        ),
    );
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ActionPlayer<A>
where
    A: Action,
{
    pub action: A,
    state: ActionState,
    run_change_callback: bool,
}

#[derive(Reflect, Default)]
enum ActionState {
    #[default]
    Inactive,
    Starting,
    Active,
    Stopping,
}

impl<A> ActionPlayer<A>
where
    A: Action,
{
    #[inline]
    pub fn new(action: A) -> Self {
        Self {
            action,
            state: ActionState::Inactive,
            run_change_callback: false,
        }
    }

    #[inline]
    pub fn start(&mut self) {
        self.state = ActionState::Starting;
    }

    #[inline]
    pub fn stop(&mut self) {
        self.state = ActionState::Stopping;
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        matches!(self.state, ActionState::Active)
    }

    #[inline]
    pub fn run_change_callback(&mut self) {
        self.run_change_callback = true;
    }
}

impl<A> Default for ActionPlayer<A>
where
    A: Action + Default,
{
    #[inline]
    fn default() -> Self {
        Self::new(A::default())
    }
}

pub type ActionId = TypeId;

pub trait Action: Any + Send + Sync {
    /// [`Action`]`s listed here will be stopped when this [`Action`] starts
    const CANCELS: &[ActionId];

    type ActionStartParam<'w, 's>: SystemParam;
    type ActionStartQuery: QueryData;
    #[allow(unused_variables)]
    fn on_action_start<'w, 's>(
        &mut self,
        query: <Self::ActionStartQuery as QueryData>::Item<'_, '_>,
        params: StaticSystemParam<Self::ActionStartParam<'w, 's>>,
    ) {
    }

    type ActionStopParam<'w, 's>: SystemParam;
    type ActionStopQuery: QueryData;
    #[allow(unused_variables)]
    fn on_action_stop<'w, 's>(
        &mut self,
        query: <Self::ActionStopQuery as QueryData>::Item<'_, '_>,
        params: StaticSystemParam<Self::ActionStopParam<'w, 's>>,
    ) {
    }

    type ActionChangeParam<'w, 's>: SystemParam;
    type ActionChangeQuery: QueryData;
    #[allow(unused_variables)]
    fn on_change<'w, 's>(
        &mut self,
        query: <Self::ActionChangeQuery as QueryData>::Item<'_, '_>,
        params: StaticSystemParam<Self::ActionChangeParam<'w, 's>>,
    ) {
    }
}

pub trait RegisterActionExt {
    fn register_action<A>(&mut self) -> &mut Self
    where
        A: Action;

    fn play_action_while_pressed<A, I>(&mut self, input: I) -> &mut Self
    where
        A: Action,
        I: Clone + Eq + Hash + Send + Sync + 'static;
}

impl RegisterActionExt for App {
    fn register_action<A>(&mut self) -> &mut Self
    where
        A: Action,
    {
        self.add_plugins(plugin::<A>)
    }

    fn play_action_while_pressed<A, I>(&mut self, input: I) -> &mut Self
    where
        A: Action,
        I: Clone + Eq + Hash + Send + Sync + 'static,
    {
        let input2 = input.clone();
        self.add_systems(
            Update,
            (|query: Query<&mut ActionPlayer<A>>| {
                for mut player in query {
                    player.start();
                }
            })
            .run_if(input_just_pressed(input2)),
        )
        .add_systems(
            Update,
            (|query: Query<&mut ActionPlayer<A>>| {
                for mut player in query {
                    player.stop();
                }
            })
            .run_if(input_just_released(input)),
        )
    }
}

fn announce_action_state_changes<A>(
    players: Query<(Entity, &ActionPlayer<A>)>,
    mut start_writer: MessageWriter<ActionStart<A>>,
    mut stop_writer: MessageWriter<ActionStop<A>>,
    mut change_writer: MessageWriter<ActionChange<A>>,
) where
    A: Action,
{
    for (player_id, player) in players {
        match player.state {
            ActionState::Starting => {
                start_writer.write(ActionStart {
                    player_id,
                    _phantom: PhantomData,
                });
            }
            ActionState::Stopping => {
                stop_writer.write(ActionStop {
                    player_id,
                    _phantom: PhantomData,
                });
            }
            _ => {}
        }
        if player.run_change_callback {
            change_writer.write(ActionChange {
                player_id,
                _phantom: PhantomData,
            });
        }
    }
}

#[derive(Message)]
struct ActionStart<A>
where
    A: Action,
{
    player_id: Entity,
    _phantom: PhantomData<A>,
}

#[derive(SystemParam)]
struct ApplyActionStartParams<'w, 's, A>
where
    A: Action,
{
    players: Query<
        'w,
        's,
        (
            &'static mut ActionPlayer<A>,
            <A as Action>::ActionStartQuery,
        ),
    >,
    reader: MessageReader<'w, 's, ActionStart<A>>,
    params: StaticSystemParam<'w, 's, <A as Action>::ActionStartParam<'static, 'static>>,
}

fn apply_action_start<A>(
    ApplyActionStartParams {
        mut players,
        mut reader,
        params,
    }: ApplyActionStartParams<'_, '_, A>,
) where
    A: Action,
{
    if let Some(ActionStart { player_id, .. }) = reader.read().next()
        && let Ok((mut player, query)) = players.get_mut(*player_id)
    {
        player.state = ActionState::Active;
        player.action.on_action_start(query, params);
    }
}

#[derive(Message)]
struct ActionStop<A>
where
    A: Action,
{
    player_id: Entity,
    _phantom: PhantomData<A>,
}

#[derive(SystemParam)]
struct ApplyActionStopParams<'w, 's, A>
where
    A: Action,
{
    players: Query<'w, 's, (&'static mut ActionPlayer<A>, <A as Action>::ActionStopQuery)>,
    reader: MessageReader<'w, 's, ActionStop<A>>,
    params: StaticSystemParam<'w, 's, <A as Action>::ActionStopParam<'static, 'static>>,
}

fn apply_action_stop<A>(
    ApplyActionStopParams {
        mut players,
        mut reader,
        params,
    }: ApplyActionStopParams<'_, '_, A>,
) where
    A: Action,
{
    if let Some(ActionStop { player_id, .. }) = reader.read().next()
        && let Ok((mut player, query)) = players.get_mut(*player_id)
    {
        player.state = ActionState::Active;
        player.action.on_action_stop(query, params);
    }
}

#[derive(Message)]
struct ActionChange<A>
where
    A: Action,
{
    player_id: Entity,
    _phantom: PhantomData<A>,
}

#[derive(SystemParam)]
struct ApplyActionChangeParams<'w, 's, A>
where
    A: Action,
{
    players: Query<
        'w,
        's,
        (
            &'static mut ActionPlayer<A>,
            <A as Action>::ActionChangeQuery,
        ),
    >,
    reader: MessageReader<'w, 's, ActionChange<A>>,
    params: StaticSystemParam<'w, 's, <A as Action>::ActionChangeParam<'static, 'static>>,
}

fn apply_action_change<A>(
    ApplyActionChangeParams {
        mut players,
        mut reader,
        params,
    }: ApplyActionChangeParams<'_, '_, A>,
) where
    A: Action,
{
    if let Some(ActionChange { player_id, .. }) = reader.read().next()
        && let Ok((mut player, query)) = players.get_mut(*player_id)
    {
        player.run_change_callback = false;
        player.action.on_change(query, params);
    }
}

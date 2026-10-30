use bevy::prelude::*;
use std::time::Duration;

use crate::ui::sidebar_menu::{ButtonPressed, SidebarMenuButton};

#[derive(Event)]
pub struct BombExploded;

#[derive(Component)]
pub struct DelayedTrigger<T: Event + Clone> {
    pub timer: Timer,
    pub event: T,
}

impl<T: Event + Clone> DelayedTrigger<T> {
    pub fn new(timer: Timer, event: T) -> DelayedTrigger<T> {
        DelayedTrigger {
            timer: timer,
            event: event,
        }
    }
}

pub struct TriggerPlugin;

impl Plugin for TriggerPlugin {
    fn build(&self, app: &mut App) {}
}

fn tick_delayed_triggers<'a, T: Event + Clone>(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DelayedTrigger<T>)>,
) where
    <T as Event>::Trigger<'a>: std::default::Default,
{
    for (entity, mut delayed) in &mut query {
        delayed.timer.tick(time.delta());

        if delayed.timer.is_finished() {
            commands.trigger(delayed.event.clone());
            commands.entity(entity).despawn();
        }
    }
}

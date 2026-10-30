use bevy::prelude::*;
use bevy_lunex::prelude::*;

use crate::{
    building::{
        Building, Guild,
        construction::{Construction, ConstructionTimer, StartConstruction},
        trainings::StartTraining,
    },
    ui::{
        buttons::{
            ButtonClicked, MenuButton, SidebarBuildingMenuButton, SidebarMenuButton,
            SidebarTrainingMenuButton,
        },
        progress_bar::{MenuProgressBar, ProgressBarState},
        sidebar_menu::*,
    },
};

pub fn setup_sidebar_training_menu(
    mut commands: Commands,
    guilds: Query<(Entity, &Guild), Added<Guild>>,
    view: Single<Entity, With<SidebarMenuView<CreationView, SidebarMenuPeople>>>,
) {
    for (entity, guild) in guilds {
        commands
            .entity(view.clone())
            .with_child(MenuButton::SidebarTrainingMenuButton(guild.clone(), entity));
    }
}

pub fn handle_sidebar_training_menu_button_click(
    event: On<ButtonClicked<SidebarTrainingMenuButton>>,
    mut commands: Commands,
) {
    commands.trigger(StartTraining(event.entity));
}
/*
pub fn handle_sidebar_menu_building_progress_started(
    mut commands: Commands,
    constructions: Query<(Entity, &Construction, &ConstructionTimer), Added<Construction>>,
    view: Single<Entity, With<SidebarMenuView<ProgressView, SidebarMenuBuilding>>>,
) {
    for (entity, construction, timer_) in constructions {
        commands
            .entity(view.clone())
            .with_child(MenuProgressBar::SidebarBuildingMenuProgressBar(
                construction.0.clone(),
                entity,
            ));
    }
}

pub fn handle_sidebar_menu_building_progressed(
    constructions: Query<&ConstructionTimer, Changed<ConstructionTimer>>,
    progress_bars: Query<(&MenuProgressBar, &mut ProgressBarState)>,
) {
    for (kind, mut state) in progress_bars {
        if let MenuProgressBar::SidebarBuildingMenuProgressBar(_, construction_entity) = kind {
            if let Ok(timer) = constructions.get(*construction_entity) {
                state.progress_percent = timer.timer.fraction()
            }
        }
    }
}

pub fn handle_sidebar_menu_building_progress_ended(
    event: On<Despawn, Construction>,
    mut commands: Commands,
    progress_bars: Query<(Entity, &MenuProgressBar)>,
) {
    for (progression_entity, progress_bar) in progress_bars {
        if let MenuProgressBar::SidebarBuildingMenuProgressBar(_, construction_entity) =
            progress_bar
        {
            if event.entity == *construction_entity {
                commands.entity(progression_entity.clone()).despawn();
            }
        }
    }
}
 */
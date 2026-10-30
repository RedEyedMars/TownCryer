use bevy::prelude::*;
use bevy_lunex::prelude::*;

use crate::{
    building::{
        Building,
        construction::{Construction, ConstructionTimer, StartConstruction},
    },
    ui::{
        IsProgressBar,
        buttons::{ButtonClicked, MenuButton, SidebarBuildingMenuButton, SidebarMenuButton},
        progress_bar::{MenuProgressBar, ProgressBarState},
        sidebar_menu::*,
    },
};

pub fn setup_sidebar_building_menu(
    mut commands: Commands,
    layout_root: Query<(Entity, &SidebarMenuView<CreationView, SidebarMenuBuilding>)>,
) {
    for (entity, view) in layout_root.iter() {
        commands.entity(entity).with_children(|ui| {
            ui.spawn(MenuButton::SidebarBuildingMenuButton(Building::House));
            ui.spawn(MenuButton::SidebarBuildingMenuButton(Building::Barracks));
            ui.spawn(MenuButton::SidebarBuildingMenuButton(Building::Market));
            ui.spawn(MenuButton::SidebarBuildingMenuButton(Building::Warehouse));
        });
    }
}

pub fn handle_sidebar_building_menu_button_click(
    event: On<ButtonClicked<SidebarBuildingMenuButton>>,
    mut commands: Commands,
) {
    commands.trigger(StartConstruction {
        building: event.button.building.clone(),
    });
}

pub fn handle_sidebar_menu_building_progress_started(
    mut commands: Commands,
    constructions: Query<(Entity, &Construction, &ConstructionTimer), Added<Construction>>,
    mut view: Single<
        (Entity, &mut SidebarMenuViewLayout),
        With<SidebarMenuView<ProgressView, SidebarMenuBuilding>>,
    >,
) {
    const PADDING_X: f32 = 2.5;
    const PADDING_Y: f32 = 2.5;
    for (entity, construction, timer_) in constructions {
        let (view, layout) = &mut *view;
        let menu_progress_bar =
            MenuProgressBar::SidebarBuildingMenuProgressBar(construction.0.clone(), entity);
        let mut x = 0.0;
        let (width, height) = menu_progress_bar.progress_bar_dimensions();
        commands.entity(*view).with_children(|ui| {
            let child = ui.spawn((
                menu_progress_bar,
                IsProgressBar,
                UiLayout::window()
                    .pos((Rl(PADDING_X) + Rl(x) + Ab(48f32), Rl(0f32)))
                    .size((Rl(width) - Ab(48f32), Ab(height)))
                    .anchor(Anchor::CENTER_LEFT)
                    .pack(),
            ));
            layout.entities.push(child.id());
        });
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
    mut layout: Single<
        &mut SidebarMenuViewLayout,
        With<SidebarMenuView<ProgressView, SidebarMenuBuilding>>,
    >,
) {
    for (progression_entity, progress_bar) in progress_bars {
        if let MenuProgressBar::SidebarBuildingMenuProgressBar(_, construction_entity) =
            progress_bar
        {
            if event.entity == *construction_entity {
                layout.entities.retain(|e| *e != progression_entity.clone());
                commands.entity(progression_entity.clone()).despawn();
            }
        }
    }
}

use std::any::Any;

use bevy::{picking::events, prelude::*};
use bevy_lunex::prelude::*;

use crate::{
    RandomSource,
    building::Building,
    ui::{IsButton, IsProgressBar, sidebar_menu::SidebarMenuViewLayout, tooltip::Tooltip},
};

#[derive(Component, PartialEq, Eq, Debug)]
pub enum MenuProgressBar {
    SidebarBuildingMenuProgressBar(Building, Entity),
    SidebarBuildingMenuProgressBar2(Building, Entity),
}

pub trait ProgressBarKind {
    fn progress_bar_kind(&self) -> MenuProgressBar;
}
#[derive(Component, PartialEq, Eq)]
pub struct SidebarBuildingMenuProgressBar {
    pub building: Building,
    pub entity: Entity,
}

impl ProgressBarKind for SidebarBuildingMenuProgressBar {
    fn progress_bar_kind(&self) -> MenuProgressBar {
        MenuProgressBar::SidebarBuildingMenuProgressBar(self.building.clone(), self.entity)
    }
}

#[derive(Component)]
pub struct ProgressBarState {
    pub progress_percent: f32,
}

pub fn menu_progress_bar(
    mut commands: Commands,
    menu_progress_bars: Query<(Entity, &MenuProgressBar), Added<MenuProgressBar>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut rng: ResMut<RandomSource>,
) {
    const PADDING_X: f32 = 2.5;
    const PADDING_Y: f32 = 2.5;

    for (menu_entity, menu_progress_bar) in menu_progress_bars {
        let mut x = 0.0;
        let (width, height) = menu_progress_bar.progress_bar_dimensions();
        commands
            .entity(menu_entity)
            .insert(ProgressBarState {
                progress_percent: 0f32,
            })
            .insert(Sprite::from_atlas_image(
                asset_server.load("ui/progress_bars.png"),
                TextureAtlas {
                    layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                        UVec2::new(48, 16),
                        7,
                        12,
                        None,
                        None,
                    )),
                    index: 8,
                },
            ))
            .with_children(|ui| {
                let mut offset = 0.;
                if let Some(icon) = menu_progress_bar.progress_bar_icon(
                    &asset_server,
                    &mut texture_atlas_layouts,
                    &mut rng,
                ) {
                    ui.spawn((
                        Pickable::IGNORE,
                        UiLayout::window()
                            .pos((Rl(15.0) - Ab(48f32), Ab(15.0)))
                            .size((Ab(24.0), Ab(24.0)))
                            .pack(),
                        icon,
                    ));
                    offset += 24.0 + 5.0;
                }
                /* here in case we want some text for progress bars
                ui.spawn((
                    Pickable::IGNORE,
                    UiLayout::window()
                        .pos((Rl(15.0) + Ab(offset), Rl(35.0)))
                        .anchor(Anchor::CENTER_LEFT)
                        .pack(),
                    UiTextSize::from(Ab(height - 24.0)),
                    Text2d::new(menu_progress_bar.progress_bar_text()),
                    TextFont {
                        font: asset_server
                            .load("fonts/Kingdom - Pixel Font v1_3.ttf")
                            .into(),
                        font_size: FontSize::Px(64.0),
                        ..Default::default()
                    },
                ));*/
            });
    }
}

pub fn handle_progress_bar_state(
    progress_bars: Query<(&ProgressBarState, &mut Sprite), Changed<ProgressBarState>>,
) {
    for (state, mut sprite) in progress_bars {
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = 13 - (6f32 * state.progress_percent) as usize;
        }
    }
}

pub fn handle_progress_bar_layout(
    layouts: Query<
        (&UiLayout, &SidebarMenuViewLayout),
        (
            Changed<SidebarMenuViewLayout>,
            Without<IsButton>,
            Without<IsProgressBar>,
        ),
    >,
    mut progress_bars: Query<&mut UiLayout, Or<(With<IsButton>, With<IsProgressBar>)>>,
) {
    for (reference, layout) in layouts {
        let reference_y = reference.get_window(UiBase::id()).unwrap().pos.get_y();
        for (i, entity) in layout.entities.iter().enumerate() {
            if let Ok(mut progress_bar) = progress_bars.get_mut(*entity) {
                *progress_bar = progress_bar
                    .get_window(UiBase::id())
                    .unwrap()
                    .y(reference_y + Ab(i as f32 * 64f32))
                    .pack();
            } else {
                panic!("somehow there is a progress bar that isn't in the layout!");
            }
        }
    }
}

impl MenuProgressBar {
    pub fn progress_bar_dimensions(&self) -> (f32, f32) {
        match self {
            MenuProgressBar::SidebarBuildingMenuProgressBar(_, _) => (95.0, 64.0),
            MenuProgressBar::SidebarBuildingMenuProgressBar2(_, _) => (95.0, 64.0),
        }
    }

    pub fn progress_bar_icon(
        &self,
        asset_server: &Res<AssetServer>,
        mut texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        mut rng: &mut ResMut<RandomSource>,
    ) -> Option<Sprite> {
        match self {
            MenuProgressBar::SidebarBuildingMenuProgressBar(building, _) => building
                .icon(asset_server, &mut texture_atlas_layouts, &mut rng.0)
                .into(),
            MenuProgressBar::SidebarBuildingMenuProgressBar2(building, _) => building
                .icon(asset_server, &mut texture_atlas_layouts, &mut rng.0)
                .into(),
        }
    }

    pub fn progress_bar_text(&self) -> impl Into<String> + '_ {
        match self {
            MenuProgressBar::SidebarBuildingMenuProgressBar(building, _) => building.name(),
            MenuProgressBar::SidebarBuildingMenuProgressBar2(building, _) => building.name(),
        }
    }
}

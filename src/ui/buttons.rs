use bevy::{picking::events, prelude::*};
use bevy_lunex::prelude::*;

use crate::{
    RandomSource,
    actor::ActorKind,
    building::{Building, Guild},
    ui::{IsButton, tooltip::Tooltip},
};

#[derive(Component, PartialEq, Eq, Debug)]
pub enum MenuButton {
    SidebarMenuButton(SidebarMenuButton),
    SidebarMenuViewButton(SidebarMenuViewButton),
    SidebarBuildingMenuButton(Building),
    SidebarTrainingMenuButton(Guild, Entity),
}

pub enum MenuButtonSprite {
    Regular,
    Select,
}

pub trait ButtonKind {
    fn button_kind(&self) -> MenuButton;
}

#[derive(Component, PartialEq, Eq, Hash, Clone, Copy, Debug, PartialOrd, Ord)]
pub enum SidebarMenuButton {
    Build,
    Roster,
}

impl ButtonKind for SidebarMenuButton {
    fn button_kind(&self) -> MenuButton {
        MenuButton::SidebarMenuButton(*self)
    }
}

#[derive(Component, PartialEq, Eq, Hash, Clone, Copy, Debug, PartialOrd, Ord)]
pub enum SidebarMenuViewButton {
    Create,
    Track,
}

impl ButtonKind for SidebarMenuViewButton {
    fn button_kind(&self) -> MenuButton {
        MenuButton::SidebarMenuViewButton(*self)
    }
}

#[derive(Component, PartialEq, Eq)]
pub struct SidebarBuildingMenuButton {
    pub building: Building,
}

impl ButtonKind for SidebarBuildingMenuButton {
    fn button_kind(&self) -> MenuButton {
        MenuButton::SidebarBuildingMenuButton(self.building.clone())
    }
}

#[derive(Component, PartialEq, Eq)]
pub struct SidebarTrainingMenuButton {
    pub kind: Guild,
    pub entity: Entity,
}

impl ButtonKind for SidebarTrainingMenuButton {
    fn button_kind(&self) -> MenuButton {
        MenuButton::SidebarTrainingMenuButton(self.kind.clone(), self.entity.clone())
    }
}

#[derive(Component, Clone)]
pub enum ButtonPressState {
    Neutral,
    Pressed,
    Toggled,
    Released,
}
#[derive(Component)]
pub enum ButtonHoverState {
    Base,
    Hovered,
}

#[derive(Event)]
pub struct ButtonClicked<T: ButtonKind> {
    pub entity: Entity,
    pub button: T,
}

pub fn menu_button(
    mut commands: Commands,
    menu_buttons: Query<(Entity, &MenuButton), Added<MenuButton>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut rng: ResMut<RandomSource>,
) {
    let mut x = 0.0;
    let mut y = 0.0;
    const PADDING_X: f32 = 2.5;
    const PADDING_Y: f32 = 2.5;

    for (menu_entity, menu_button) in menu_buttons {
        let (width, height) = menu_button.button_dimensions();
        let mut button_entity = commands.entity(menu_entity);
        button_entity
            .insert(IsButton)
            .insert(ButtonPressState::Neutral)
            .insert(ButtonHoverState::Base)
            .insert(
                UiLayout::window()
                    .pos((Rl(PADDING_X) + Rl(x), Ab(y) + Ab(height / 2.0)))
                    .size((Rl(width), Ab(height)))
                    .anchor(Anchor::CENTER_LEFT)
                    .pack(),
            )
            .with_children(|ui| {
                let mut offset = 0.;
                match menu_button.button_sprite() {
                    MenuButtonSprite::Regular => {
                        if let Some(icon) = menu_button.button_icon(
                            &asset_server,
                            &mut texture_atlas_layouts,
                            &mut rng,
                        ) {
                            ui.spawn((
                                Pickable::IGNORE,
                                UiLayout::window()
                                    .pos((Rl(20.0), Ab(10.0)))
                                    .size((Ab(24.0), Ab(24.0)))
                                    .pack(),
                                icon,
                            ));
                            offset += 24.0;
                        }
                    }
                    MenuButtonSprite::Select => {
                        offset += 24.0;
                    }
                }
                ui.spawn((
                    Pickable::IGNORE,
                    UiLayout::window()
                        .pos((Rl(50.0) + Ab(offset), Rl(35.0)))
                        .anchor(Anchor::CENTER)
                        .pack(),
                    UiTextSize::from(Ab(height - 24.0)),
                    Text2d::new(menu_button.button_text()),
                    TextFont {
                        font: asset_server
                            .load("fonts/Kingdom - Pixel Font v1_3.ttf")
                            .into(),
                        font_size: FontSize::Px(64.0),
                        ..Default::default()
                    },
                ));
            })
            .observe(
                |event: On<Pointer<Press>>, mut states: Query<&mut ButtonPressState>| {
                    let mut pressed_state = states
                        .get_mut(event.entity)
                        .expect("Failed to get sprite for entity");
                    *pressed_state = match *pressed_state {
                        ButtonPressState::Toggled => ButtonPressState::Toggled,
                        _ => ButtonPressState::Pressed,
                    };
                },
            )
            .observe(
                |event: On<Pointer<Release>>, mut states: Query<&mut ButtonPressState>| {
                    let mut pressed_state = states
                        .get_mut(event.entity)
                        .expect("Failed to get sprite for entity");
                    *pressed_state = match *pressed_state {
                        ButtonPressState::Neutral => ButtonPressState::Neutral,
                        ButtonPressState::Toggled => ButtonPressState::Toggled,
                        _ => ButtonPressState::Released,
                    };
                },
            )
            .observe(
                |event: On<Pointer<Out>>, mut states: Query<&mut ButtonHoverState>| {
                    let mut hovered_state =
                        states.get_mut(event.entity).expect("Failed to get entity");
                    *hovered_state = ButtonHoverState::Base;
                },
            )
            .observe(
                |event: On<Pointer<Move>>, mut states: Query<&mut ButtonHoverState>| {
                    let mut hovered_state =
                        states.get_mut(event.entity).expect("Failed to get entity");
                    *hovered_state = ButtonHoverState::Hovered;
                },
            )
            .observe(
                |event: On<Pointer<Click>>,
                 mut commands: Commands,
                 buttons: Query<(Entity, &MenuButton)>| {
                    let (entity, clicked_button) = buttons
                        .get_inner(event.entity)
                        .expect("Failed to get button for entity");
                    match clicked_button {
                        MenuButton::SidebarMenuButton(kind) => commands.trigger(ButtonClicked {
                            entity,
                            button: *kind,
                        }),
                        MenuButton::SidebarMenuViewButton(kind) => {
                            commands.trigger(ButtonClicked {
                                entity,
                                button: *kind,
                            })
                        }
                        MenuButton::SidebarBuildingMenuButton(building) => {
                            commands.trigger(ButtonClicked {
                                entity,
                                button: SidebarBuildingMenuButton {
                                    building: building.clone(),
                                },
                            })
                        }
                        MenuButton::SidebarTrainingMenuButton(kind, guild_entity) => commands
                            .trigger(ButtonClicked {
                                entity,
                                button: SidebarTrainingMenuButton {
                                    kind: kind.clone(),
                                    entity: guild_entity.clone(),
                                },
                            }),
                    }
                },
            );
        match menu_button.button_sprite() {
            MenuButtonSprite::Regular => button_entity.insert(Sprite::from_atlas_image(
                asset_server.load("ui/button_2.png"),
                TextureAtlas {
                    layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                        UVec2::new(64, 32),
                        1,
                        2,
                        None,
                        None,
                    )),
                    index: 0,
                },
            )),
            MenuButtonSprite::Select => button_entity
                .insert(Sprite::from_atlas_image(
                    asset_server.load("ui/2x1.png"),
                    TextureAtlas {
                        layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                            UVec2::new(1, 1),
                            2,
                            1,
                            None,
                            None,
                        )),
                        index: 0,
                    },
                ))
                .with_children(|ui| {
                    ui.spawn((
                        UiLayout::window()
                            .pos((Rl(10.0), Rl(-15.0)))
                            .size((Ab(64.0), Ab(64.0)))
                            .pack(),
                        IsSubButton,
                        ButtonPressState::Neutral,
                        ButtonHoverState::Base,
                        Sprite::from_atlas_image(
                            asset_server.load("ui/select.png"),
                            TextureAtlas {
                                layout: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
                                    UVec2::new(64, 32),
                                    1,
                                    2,
                                    None,
                                    None,
                                )),
                                index: 0,
                            },
                        ),
                    ))
                    .with_children(|ui| {
                        if let Some(icon) = menu_button.button_icon(
                            &asset_server,
                            &mut texture_atlas_layouts,
                            &mut rng,
                        ) {
                            ui.spawn((
                                Pickable::IGNORE,
                                UiLayout::window()
                                    .pos((Rl(25.0), Rl(40.0)))
                                    .size((Ab(16.0), Ab(16.0)))
                                    .anchor(Anchor::CENTER)
                                    .pack(),
                                icon,
                            ));
                        }
                    });
                }),
        };
        x += width + PADDING_X;
        if x > 90.0 {
            x = 0.0;
            y += height + PADDING_Y;
        }
    }
}

#[derive(Component)]
pub struct IsSubButton;

pub fn handle_button_state(
    buttons: Query<
        (&ButtonPressState, &Children, &mut Sprite),
        (Without<IsSubButton>, Changed<ButtonPressState>),
    >,
    mut transforms: Query<&mut UiLayout, Or<(With<Text2d>, With<Sprite>)>>,
    mut sub_buttons: Query<&mut ButtonPressState, With<IsSubButton>>,
) {
    for (state, children, mut sprite) in buttons {
        match state {
            ButtonPressState::Neutral => {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = 0;
                }
                for child in children.iter() {
                    if let Ok(mut sub_button_state) = sub_buttons.get_mut(child) {
                        *sub_button_state = state.clone();
                    }
                }
            }
            ButtonPressState::Pressed => {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = 1;
                }
                let mut has_sub_button = false;
                for child in children.iter() {
                    if let Ok(mut sub_button_state) = sub_buttons.get_mut(child) {
                        *sub_button_state = state.clone();
                        has_sub_button = true;
                    }
                }
                if !has_sub_button {
                    for child in children.iter() {
                        if let Ok(mut transform) = transforms.get_mut(child) {
                            *transform = transform
                                .get_mut_window(UiBase::id())
                                .unwrap()
                                .y(transform.get_window(UiBase::id()).unwrap().pos.get_y()
                                    + Ab(6f32))
                                .pack();
                        }
                    }
                }
            }
            ButtonPressState::Released => {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = 0;
                }
                let mut has_sub_button = false;
                for child in children.iter() {
                    if let Ok(mut sub_button_state) = sub_buttons.get_mut(child) {
                        *sub_button_state = state.clone();
                        has_sub_button = true;
                    }
                }
                if !has_sub_button {
                    for child in children.iter() {
                        if let Ok(mut transform) = transforms.get_mut(child) {
                            *transform = transform
                                .get_mut_window(UiBase::id())
                                .unwrap()
                                .y(transform.get_window(UiBase::id()).unwrap().pos.get_y()
                                    - Ab(6f32))
                                .pack();
                        }
                    }
                }
            }
            ButtonPressState::Toggled => {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = 1;
                }
                for child in children.iter() {
                    if let Ok(mut sub_button_state) = sub_buttons.get_mut(child) {
                        *sub_button_state = state.clone();
                    }
                }
            }
        }
    }
}

pub fn handle_sub_button_state(
    buttons: Query<
        (&ButtonPressState, &Children, &mut Sprite),
        (With<IsSubButton>, Changed<ButtonPressState>),
    >,
    mut transforms: Query<&mut UiLayout, Or<(With<Text2d>, With<Sprite>)>>,
) {
    for (state, children, mut sprite) in buttons {
        match state {
            ButtonPressState::Neutral => {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = 0;
                }
            }
            ButtonPressState::Pressed => {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = 1;
                }
                for child in children.iter() {
                    if let Ok(mut transform) = transforms.get_mut(child) {
                        *transform = transform
                            .get_mut_window(UiBase::id())
                            .unwrap()
                            .y(transform.get_window(UiBase::id()).unwrap().pos.get_y() + Ab(6f32))
                            .pack();
                    }
                }
            }
            ButtonPressState::Released => {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = 0;
                }
                for child in children.iter() {
                    if let Ok(mut transform) = transforms.get_mut(child) {
                        *transform = transform
                            .get_mut_window(UiBase::id())
                            .unwrap()
                            .y(transform.get_window(UiBase::id()).unwrap().pos.get_y() - Ab(6f32))
                            .pack();
                    }
                }
            }
            ButtonPressState::Toggled => {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = 1;
                }
            }
        }
    }
}

impl MenuButton {
    pub fn button_dimensions(&self) -> (f32, f32) {
        match self {
            MenuButton::SidebarMenuButton(_) => (45.0, 64.0),
            MenuButton::SidebarMenuViewButton(_) => (45.0, 64.0),
            MenuButton::SidebarBuildingMenuButton(_) => (95.0, 64.0),
            MenuButton::SidebarTrainingMenuButton(_, _) => (95.0, 64.0),
        }
    }

    pub fn button_icon(
        &self,
        asset_server: &Res<AssetServer>,
        mut texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        mut rng: &mut ResMut<RandomSource>,
    ) -> Option<Sprite> {
        match self {
            MenuButton::SidebarMenuButton(SidebarMenuButton::Build) => None,
            MenuButton::SidebarMenuButton(SidebarMenuButton::Roster) => None,

            MenuButton::SidebarMenuViewButton(SidebarMenuViewButton::Create) => None,
            MenuButton::SidebarMenuViewButton(SidebarMenuViewButton::Track) => None,

            MenuButton::SidebarBuildingMenuButton(building) => building
                .icon(asset_server, &mut texture_atlas_layouts, &mut rng.0)
                .into(),

            MenuButton::SidebarTrainingMenuButton(guild, _) => guild
                .training_template
                .end_kind
                .icon(asset_server, &mut texture_atlas_layouts, &mut rng.0)
                .into(),
        }
    }

    pub fn button_text(&self) -> impl Into<String> + '_ {
        match self {
            MenuButton::SidebarMenuButton(SidebarMenuButton::Build) => "Build",
            MenuButton::SidebarMenuButton(SidebarMenuButton::Roster) => "Roster",

            MenuButton::SidebarMenuViewButton(SidebarMenuViewButton::Create) => "Create",
            MenuButton::SidebarMenuViewButton(SidebarMenuViewButton::Track) => "Track",

            MenuButton::SidebarBuildingMenuButton(building) => building.name(),
            MenuButton::SidebarTrainingMenuButton(guild, _) => {
                guild.training_template.end_kind.name()
            }
        }
    }

    pub fn button_sprite(&self) -> MenuButtonSprite {
        match self {
            MenuButton::SidebarTrainingMenuButton(_, _) => MenuButtonSprite::Select,
            _ => MenuButtonSprite::Regular,
        }
    }
}

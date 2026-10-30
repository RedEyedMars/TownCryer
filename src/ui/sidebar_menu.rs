use std::marker::PhantomData;

use bevy::{ecs::query::QueryEntityError, picking::events, prelude::*};
use bevy_lunex::prelude::*;

use crate::{
    RandomSource,
    building::Building,
    ui::{
        IsButton,
        buttons::{
            ButtonClicked, ButtonPressState, MenuButton, SidebarMenuButton, SidebarMenuViewButton,
        },
        tooltip::Tooltip,
    },
};

pub struct SidebarMenuPlugin;

impl Plugin for SidebarMenuPlugin {
    fn build(&self, app: &mut App) {}
}

#[derive(Component)]
pub struct SidebarMenu;

pub trait SidebarMenuOption {
    fn toggle_view_button(&self) -> SidebarMenuButton;
}
pub struct SidebarMenuBuilding;
impl SidebarMenuOption for SidebarMenuBuilding {
    fn toggle_view_button(&self) -> SidebarMenuButton {
        SidebarMenuButton::Build
    }
}
pub struct SidebarMenuPeople;
impl SidebarMenuOption for SidebarMenuPeople {
    fn toggle_view_button(&self) -> SidebarMenuButton {
        SidebarMenuButton::Roster
    }
}

pub struct CreationView;
pub struct ProgressView;
pub struct RosterView;

#[derive(Component)]
pub struct SidebarMenuView<T, U: SidebarMenuOption> {
    pub kind: U,
    _marker: PhantomData<T>,
}

#[derive(Component)]
pub struct SidebarMenuViewLayout {
    pub entities: Vec<Entity>,
}

pub trait SidebarMenuViewTrait {
    fn as_menu_buttons(&self) -> (MenuButton, MenuButton);
}

impl SidebarMenuViewTrait for &SidebarMenuView<CreationView, SidebarMenuBuilding> {
    fn as_menu_buttons(&self) -> (MenuButton, MenuButton) {
        (
            MenuButton::SidebarMenuButton(SidebarMenuButton::Build),
            MenuButton::SidebarMenuViewButton(SidebarMenuViewButton::Create),
        )
    }
}

impl SidebarMenuViewTrait for &SidebarMenuView<ProgressView, SidebarMenuBuilding> {
    fn as_menu_buttons(&self) -> (MenuButton, MenuButton) {
        (
            MenuButton::SidebarMenuButton(SidebarMenuButton::Build),
            MenuButton::SidebarMenuViewButton(SidebarMenuViewButton::Track),
        )
    }
}
impl SidebarMenuViewTrait for &SidebarMenuView<CreationView, SidebarMenuPeople> {
    fn as_menu_buttons(&self) -> (MenuButton, MenuButton) {
        (
            MenuButton::SidebarMenuButton(SidebarMenuButton::Roster),
            MenuButton::SidebarMenuViewButton(SidebarMenuViewButton::Create),
        )
    }
}

impl SidebarMenuViewTrait for &SidebarMenuView<ProgressView, SidebarMenuPeople> {
    fn as_menu_buttons(&self) -> (MenuButton, MenuButton) {
        (
            MenuButton::SidebarMenuButton(SidebarMenuButton::Roster),
            MenuButton::SidebarMenuViewButton(SidebarMenuViewButton::Track),
        )
    }
}

pub fn setup_sidebar_menu(
    mut commands: Commands,
    mut layout_root: Query<Entity, With<UiLayoutRoot>>,
    asset_server: Res<AssetServer>,
) {
    let root_entity = layout_root.single_mut().expect("Root entity must be setup");
    commands.entity(root_entity).with_children(|ui| {
        ui.spawn((
            Name::new("Sidebar Menu Root"),
            UiLayout::boundary()
                .pos1((Rl(80.0), Rl(0.0)))
                .pos2((Rl(100.0), Rl(100.0)))
                .pack(),
        ))
        .with_children(|ui| {
            ui.spawn((
                Name::new("Sidebar Menu"),
                SidebarMenu,
                UiLayout::solid().size((Rl(100.0), Rl(100.0))).pack(),
                Sprite::from_image(asset_server.load("ui/menu.png")),
            ));
            ui.spawn(Tooltip);
        });
    });
}

pub fn setup_castle_menu(
    mut commands: Commands,
    mut layout_root: Query<Entity, With<SidebarMenu>>,
    asset_server: Res<AssetServer>,
) {
    let castle_menu_entity = layout_root.single_mut().expect("Root entity must be setup");
    commands.entity(castle_menu_entity).with_children(|ui| {
        ui.spawn((
            Name::new("Castle Menu"),
            SidebarMenu,
            UiLayout::window()
                .anchor(Anchor::CENTER_LEFT)
                .pos((Rl(5.0), Rl(5.0)))
                .size((Rl(90.0), Ab(64.0)))
                .pack(),
            Sprite::from_image(asset_server.load("ui/banner.png")),
        ))
        .with_children(|ui| {
            ui.spawn((
                UiLayout::window()
                    .pos((Rl(50.), Rl(55.0)))
                    .anchor(Anchor::CENTER)
                    .pack(),
                UiColor::new(vec![(UiBase::id(), Srgba::new(1., 1., 1., 1.))]),
                UiHover::new().forward_speed(20.0).backward_speed(4.0),
                UiTextSize::from(Rh(50.0)),
                Text2d::new("Castle"),
                TextFont {
                    font: asset_server
                        .load("fonts/Kingdom - Pixel Font v1_3.ttf")
                        .into(),
                    font_size: FontSize::Px(64.0),
                    ..Default::default()
                },
                // Make sure it does not cover the bounding zone of parent
                Pickable::IGNORE,
            ));
        });

        ui.spawn((
            Name::new("Button area"),
            UiLayout::window()
                .anchor(Anchor::CENTER_LEFT)
                .pos((Rl(5.0), Rl(40.0)))
                .size((Rl(90.0), Rl(24.)))
                .pack(),
        ))
        .with_children(|ui| {
            ui.spawn((
                MenuButton::SidebarMenuButton(SidebarMenuButton::Build),
                //SidebarMenuButton::Build,
            ));
            ui.spawn((
                MenuButton::SidebarMenuButton(SidebarMenuButton::Roster),
                //SidebarMenuButton::Roster,
            ));
        });

        ui.spawn((
            Name::new("Button View area"),
            UiLayout::window()
                .anchor(Anchor::CENTER_LEFT)
                .pos((Rl(5.0), Rl(40.0)))
                .size((Rl(90.0), Rl(24.)))
                .pack(),
        ))
        .with_children(|ui| {
            ui.spawn((
                MenuButton::SidebarMenuViewButton(SidebarMenuViewButton::Create),
                //SidebarMenuButton::Build,
            ));
            ui.spawn((
                MenuButton::SidebarMenuViewButton(SidebarMenuViewButton::Track),
                //SidebarMenuButton::Roster,
            ));
        });

        ui.spawn((
            Name::new("Building - creation"),
            UiLayout::window()
                .anchor(Anchor::CENTER_LEFT)
                .pos((Rl(5.0), Rl(45.0)))
                .size((Rl(90.0), Rl(32.)))
                .pack(),
            Visibility::Hidden,
            SidebarMenuView {
                kind: SidebarMenuBuilding,
                _marker: PhantomData::<CreationView>::default(),
            },
        ));

        ui.spawn((
            Name::new("Building - progress"),
            UiLayout::window()
                .anchor(Anchor::CENTER_LEFT)
                .pos((Rl(5.0), Rl(50.0)))
                .size((Rl(90.0), Rl(32.)))
                .pack(),
            Visibility::Hidden,
            SidebarMenuView {
                kind: SidebarMenuBuilding,
                _marker: PhantomData::<ProgressView>::default(),
            },
            SidebarMenuViewLayout { entities: vec![] },
        ));

        ui.spawn((
            Name::new("People - creation"),
            UiLayout::window()
                .anchor(Anchor::CENTER_LEFT)
                .pos((Rl(5.0), Rl(65.0)))
                .size((Rl(90.0), Rl(32.)))
                .pack(),
            Visibility::Hidden,
            SidebarMenuView {
                kind: SidebarMenuPeople,
                _marker: PhantomData::<CreationView>::default(),
            },
        ));

        ui.spawn((
            Name::new("People - progress"),
            UiLayout::window()
                .anchor(Anchor::CENTER_LEFT)
                .pos((Rl(5.0), Rl(60.0)))
                .size((Rl(90.0), Rl(32.)))
                .pack(),
            Visibility::Hidden,
            SidebarMenuView {
                kind: SidebarMenuPeople,
                _marker: PhantomData::<ProgressView>::default(),
            },
        ));
    });
}

pub fn handle_sidebar_menu_button_click(
    event: On<ButtonClicked<SidebarMenuButton>>,
    mut button_states: Query<(&MenuButton, &mut ButtonPressState)>,

    views1: Single<(Entity, &SidebarMenuView<CreationView, SidebarMenuBuilding>)>,
    views2: Single<(Entity, &SidebarMenuView<ProgressView, SidebarMenuBuilding>)>,
    views3: Single<(Entity, &SidebarMenuView<CreationView, SidebarMenuPeople>)>,
    views4: Single<(Entity, &SidebarMenuView<ProgressView, SidebarMenuPeople>)>,
    mut visibilities: Query<&mut Visibility>,
) {
    for (menu_button, mut button_state) in button_states.iter_mut() {
        if let (MenuButton::SidebarMenuButton(_), ButtonPressState::Toggled) =
            (menu_button, button_state.clone())
        {
            *button_state = ButtonPressState::Released;
        }
    }
    if let Ok((_, mut button_state)) = button_states.get_mut(event.entity) {
        *button_state = ButtonPressState::Toggled;
    }
    let (vis1, view1) = *views1;
    let (vis2, view2) = *views2;
    let (vis3, view3) = *views3;
    let (vis4, view4) = *views4;
    handle_sidebar_menu_view_toggle(
        &mut button_states,
        vec![
            (vis1, view1.as_menu_buttons()),
            (vis2, view2.as_menu_buttons()),
            (vis3, view3.as_menu_buttons()),
            (vis4, view4.as_menu_buttons()),
        ],
        &mut visibilities,
    );
}

pub fn handle_sidebar_menu_view_button_click(
    event: On<ButtonClicked<SidebarMenuViewButton>>,
    mut button_states: Query<(&MenuButton, &mut ButtonPressState)>,

    views1: Single<(Entity, &SidebarMenuView<CreationView, SidebarMenuBuilding>)>,
    views2: Single<(Entity, &SidebarMenuView<ProgressView, SidebarMenuBuilding>)>,
    views3: Single<(Entity, &SidebarMenuView<CreationView, SidebarMenuPeople>)>,
    views4: Single<(Entity, &SidebarMenuView<ProgressView, SidebarMenuPeople>)>,
    mut visibilities: Query<&mut Visibility>,
) {
    for (menu_button, mut button_state) in button_states.iter_mut() {
        if let (MenuButton::SidebarMenuViewButton(_), ButtonPressState::Toggled) =
            (menu_button, button_state.clone())
        {
            *button_state = ButtonPressState::Released;
        }
    }
    if let Ok((_, mut button_state)) = button_states.get_mut(event.entity) {
        *button_state = ButtonPressState::Toggled;
    }
    let (vis1, view1) = *views1;
    let (vis2, view2) = *views2;
    let (vis3, view3) = *views3;
    let (vis4, view4) = *views4;
    handle_sidebar_menu_view_toggle(
        &mut button_states,
        vec![
            (vis1, view1.as_menu_buttons()),
            (vis2, view2.as_menu_buttons()),
            (vis3, view3.as_menu_buttons()),
            (vis4, view4.as_menu_buttons()),
        ],
        &mut visibilities,
    );
}

pub fn handle_sidebar_menu_view_toggle(
    button_states: &Query<(&MenuButton, &mut ButtonPressState)>,
    views: Vec<(Entity, (MenuButton, MenuButton))>,
    mut visibilities: &mut Query<&mut Visibility>,
) {
    let mut menu_button: Option<&MenuButton> = None;
    let mut view_button: Option<&MenuButton> = None;
    for button_state in button_states.iter() {
        if let (MenuButton::SidebarMenuButton(_), ButtonPressState::Toggled) = button_state {
            menu_button = Some(button_state.0);
        }
        if let (MenuButton::SidebarMenuViewButton(_), ButtonPressState::Toggled) = button_state {
            view_button = Some(button_state.0);
        }
    }

    info!("Toggled buttons: {:?} {:?}", menu_button, view_button);
    for (visibility, (view_eq_button_1, view_eq_button_2)) in views {
        if let Ok(mut vis) = visibilities.get_mut(visibility) {
            if let Some(menu_button_1) = menu_button {
                if let Some(view_button_2) = view_button {
                    if *menu_button_1 == view_eq_button_1 && *view_button_2 == view_eq_button_2 {
                        *vis = Visibility::Visible;
                        continue;
                    }
                }
            }
            *vis = Visibility::Hidden;
        }
    }
}

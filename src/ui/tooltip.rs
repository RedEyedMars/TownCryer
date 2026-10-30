use bevy::{
    prelude::*,
    text::{LineHeight, TextBounds},
};
use bevy_lunex::{UiLayoutTypeWindow, prelude::*};

use crate::ui::{buttons::ButtonHoverState, buttons::MenuButton};

#[derive(Component)]
pub struct Tooltip;

#[derive(Component)]
pub struct TooltipText;

pub fn setup_tooltip(
    mut commands: Commands,
    mut layout_root: Query<Entity, With<Tooltip>>,
    asset_server: Res<AssetServer>,
) {
    let tooltip_entity = layout_root.single_mut().expect("Tooltip entity not found");
    commands
        .entity(tooltip_entity)
        .insert(Name::new("Tooltip"))
        .insert(
            UiLayout::window()
                .anchor(Anchor::CENTER_RIGHT)
                .pos((Rl(0.0), Rl(0.0)))
                .size((Rl(100.0), Rl(30.)))
                .pack(),
        )
        .insert(Sprite::from_image(asset_server.load("ui/menu.png")))
        .insert(Visibility::Hidden)
        .with_children(|ui| {
            ui.spawn((
                Name::new("Tooltip Text"),
                UiLayout::window()
                    .anchor(Anchor::CENTER)
                    .pos((Rl(50.0), Rl(35.0)))
                    .size((Rl(70.0), Rl(60.0)))
                    .pack(),
                UiTextSize::from(Ab(12.0)),
                TooltipText,
                Text2d::new("the brown fox jumped over the brown fox over the green log the brown fox jumped over the brown fox over the green log the brown fox jumped over the brown fox over the green log"),
                TextBounds::new(100.0, 60.0),
                TextFont {
                    font: asset_server.load("fonts/m5x7.ttf").into(),
                    font_size: FontSize::Px(16.0),
                    ..Default::default()
                },
                LineHeight::RelativeToFont(0.6),
                TextLayout::justify(Justify::Left),
            ));
        });
}

pub fn handle_sidebar_building_menu_hover_tooltip(
    mut tooltip_text: Single<&mut Text2d, With<TooltipText>>,
    mut tooltip: Single<(&mut Visibility, &mut UiLayout), With<Tooltip>>,
    button_positions: Query<
        (&UiLayout, &MenuButton, &ButtonHoverState),
        (Changed<ButtonHoverState>, Without<Tooltip>),
    >,
) {
    for (button_position, button, hover_state) in button_positions.iter() {
        match hover_state {
            ButtonHoverState::Hovered => {
                let building_kind = match button {
                    MenuButton::SidebarBuildingMenuButton(building) => building,
                    _ => continue,
                };

                *tooltip.1 = tooltip
                    .1
                    .get_mut_window(UiBase::id())
                    .unwrap()
                    .y(button_position
                        .get_window(UiBase::id())
                        .unwrap()
                        .pos
                        .get_y()
                        + Rl(30f32))
                    .pack();
                tooltip_text.0 = String::from(building_kind.tooltip());
                *tooltip.0 = Visibility::Visible;
            }
            _ => {
                *tooltip.0 = Visibility::Hidden;
            }
        }
    }
}

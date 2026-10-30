use bevy::prelude::*;
use bevy_lunex::prelude::*;

pub mod buttons;
pub mod progress_bar;
pub mod sidebar_building_menu;
pub mod sidebar_menu;
pub mod sidebar_training_menu;
pub mod tooltip;

#[derive(Component)]
pub struct IsButton;

#[derive(Component)]
pub struct IsProgressBar;

#[derive(Component)]
pub struct IsButtonText;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                menu_setup,
                sidebar_menu::setup_sidebar_menu,
                sidebar_menu::setup_castle_menu,
                tooltip::setup_tooltip,
                sidebar_building_menu::setup_sidebar_building_menu,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                (
                    buttons::menu_button,
                    tooltip::handle_sidebar_building_menu_hover_tooltip,
                    buttons::handle_button_state,
                    buttons::handle_sub_button_state,
                    progress_bar::menu_progress_bar,
                    progress_bar::handle_progress_bar_state,
                    sidebar_building_menu::handle_sidebar_menu_building_progress_started,
                    sidebar_building_menu::handle_sidebar_menu_building_progressed,
                    sidebar_training_menu::setup_sidebar_training_menu,
                ),
                progress_bar::handle_progress_bar_layout,
            )
                .chain(),
        )
        .add_plugins(sidebar_menu::SidebarMenuPlugin)
        .add_observer(sidebar_menu::handle_sidebar_menu_button_click)
        .add_observer(sidebar_menu::handle_sidebar_menu_view_button_click)
        .add_observer(sidebar_building_menu::handle_sidebar_building_menu_button_click)
        .add_observer(sidebar_building_menu::handle_sidebar_menu_building_progress_ended)
        .add_observer(sidebar_training_menu::handle_sidebar_training_menu_button_click);
    }
}

pub fn menu_setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Root"),
        UiLayoutRoot::new_2d(),
        UiFetchFromCamera::<0>,
    ));
}

pub fn menu_follow_camera(
    mut query: Query<(&mut Transform, &UiLayoutRoot), Without<Camera2d>>,
    camera_query: Query<&Transform, With<Camera2d>>,
) {
    if let Ok(camera_transform) = camera_query.single() {
        for (mut transform, _) in query.iter_mut() {
            transform.translation = camera_transform.translation;
            transform.translation.z = 100.0; // Ensure the menu is above the camera
        }
    }
}

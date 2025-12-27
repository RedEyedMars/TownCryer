use bevy::{
    prelude::*,
    window::{Window, WindowPlugin, WindowResolution},
};
use bevy_ecs_tiled::prelude::*;
use bevy_lunex::{UiLunexDebugPlugin, UiLunexPlugins, UiSourceCamera};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::time::SystemTime;

pub mod actor;
pub mod building;
pub mod castle;
pub mod common;
pub mod environment;
pub mod sprite_animation;
pub mod ui;
pub mod world;

use crate::{common::Direction, ui::InspectHandle};

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Town Cryer".to_string(),
                        resolution: WindowResolution::new(1024., 768.)
                            .with_scale_factor_override(1.),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(UiLunexPlugins)
        //.add_plugins(UiLunexDebugPlugin::<0, 0>)
        .add_plugins(TiledMapPlugin::default())
        .add_plugins(actor::actions::ActionPlugin)
        .add_plugins(actor::thinking::ThinkingPlugin)
        .add_plugins(world::plugin::WorldPlugin)
        .add_systems(Startup, (setup, ui::setup_menus))
        .add_systems(First, update_cursor_pos)
        .add_systems(Update, debug_actor_commands)
        .add_systems(
            Update,
            (debug_world_commands, ui::menu_follow_camera).chain(),
        )
        .add_observer(ui::observer_hover_button)
        .add_observer(ui::observer_hover_button_text)
        .add_systems(
            Update,
            (
                building::spawn_villagers,
                building::construction::update_construction,
            ),
        )
        .add_systems(
            Update,
            (
                ui::barracks::set_members_and_trainees_when_guild_changes,
                ui::barracks::set_members_and_trainees_when_inspect_handle_changes,
                ui::barracks::set_members_and_trainees_when_training_changes,
            ),
        )
        .add_systems(FixedUpdate, (sprite_animation::animate_sprite,))
        .run();
}

#[derive(Resource)]
pub struct RandomSource(ChaCha8Rng);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut seeded_rng = ChaCha8Rng::seed_from_u64(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    );

    commands.spawn((Camera2d, UiSourceCamera::<0>));

    actor::spawns::spawn_enemy_actor(
        &mut commands,
        &asset_server,
        &mut texture_atlas_layouts,
        Transform::from_translation(Vec3::new(100.0, 100.0, 0.0)),
    );

    commands.insert_resource(RandomSource(seeded_rng));
    commands.insert_resource(sprite_animation::AnimationTimer(Timer::from_seconds(
        0.1,
        TimerMode::Repeating,
    )));
    commands.insert_resource(CursorPos::default());
}

fn debug_actor_commands(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut sprite_animation::AnimationState, &mut Direction)>,
) {
    if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
        for (mut state, mut direction) in query.iter_mut() {
            *direction = Direction::North;
            if let sprite_animation::AnimationState::Walk(_) = *state {
            } else {
                *state = sprite_animation::AnimationState::Walk(0);
            }
        }
    } else if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
        for (mut state, mut direction) in query.iter_mut() {
            *direction = Direction::South;
            if let sprite_animation::AnimationState::Walk(_) = *state {
            } else {
                *state = sprite_animation::AnimationState::Walk(0);
            }
        }
    } else if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
        for (mut state, mut direction) in query.iter_mut() {
            *direction = Direction::West;
            if let sprite_animation::AnimationState::Walk(_) = *state {
            } else {
                *state = sprite_animation::AnimationState::Walk(0);
            }
        }
    } else if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
        for (mut state, mut direction) in query.iter_mut() {
            *direction = Direction::East;
            if let sprite_animation::AnimationState::Walk(_) = *state {
            } else {
                *state = sprite_animation::AnimationState::Walk(0);
            }
        }
    }
    if keyboard_input.just_pressed(KeyCode::Space) {
        for (mut state, _) in query.iter_mut() {
            *state =
                sprite_animation::AnimationState::Attack(Timer::from_seconds(0.1, TimerMode::Once));
        }
    }
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        for (mut state, _) in query.iter_mut() {
            *state = sprite_animation::AnimationState::SpecialAction(Timer::from_seconds(
                0.1,
                TimerMode::Once,
            ));
        }
    }
    if keyboard_input.just_pressed(KeyCode::KeyF) {
        for (mut state, _) in query.iter_mut() {
            *state = sprite_animation::AnimationState::Idle;
        }
    }
}

pub fn debug_world_commands(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera: Single<(&mut Transform, &Camera2d)>,
) {
    let transform = &mut camera.0;
    if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
        transform.translation.y += 16.0;
    } else if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
        transform.translation.y -= 16.0;
    } else if keyboard_input.pressed(KeyCode::ArrowLeft) || keyboard_input.pressed(KeyCode::KeyA) {
        transform.translation.x -= 16.0;
    } else if keyboard_input.pressed(KeyCode::ArrowRight) || keyboard_input.pressed(KeyCode::KeyD) {
        transform.translation.x += 16.0;
    }
}

#[derive(Resource)]
pub struct CursorPos(Vec2);
impl Default for CursorPos {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self(Vec2::new(-1000.0, -1000.0))
    }
}

pub fn update_cursor_pos(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    for cursor_moved in cursor_moved_events.read() {
        for (cam_t, cam) in camera_q.iter() {
            if let Ok(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                *cursor_pos = CursorPos(pos);
            }
        }
    }
}

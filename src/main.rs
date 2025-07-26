use bevy::{
    platform::collections::HashMap,
    prelude::*,
    window::{Window, WindowPlugin, WindowResolution},
};
use bevy_ecs_tiled::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::time::SystemTime;

pub mod actor;
pub mod building;
pub mod common;
pub mod environment;
pub mod sprite_animation;
pub mod tiled_plugin;
pub mod world;

use crate::{common::Direction, tiled_plugin::TiledMapWithTextures};

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
        .add_plugins(TiledMapPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (debug_actor_commands, debug_world_commands))
        .add_systems(
            Update,
            (tiled_plugin::load, world::WorldWithTiles::build_world),
        )
        .add_systems(
            Update,
            (
                actor::fix_idlers,
                actor::fix_thinkers,
                actor::move_attacking_actors,
                actor::attacking_actors_act,
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
    let seeded_rng = ChaCha8Rng::seed_from_u64(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    );

    commands.spawn(Camera2d);

    let map_handle: Handle<TiledMap> = asset_server.load("world.tmx");

    commands.spawn(TilemapAnchor::default());
    commands.spawn(TilemapRenderSettings::default());
    commands.spawn(TiledMapLayerZOffset::default());
    commands.insert_resource(world::WorldWithTiles {
        width: 256,
        height: 256,
        loaded: false,
        tiles: vec![world::TileType::None; 256 * 256],
    });
    commands.insert_resource(TiledMapWithTextures {
        loaded: false,
        handle: map_handle,
        map: None,
        tilemap_textures: HashMap::default(),
        tile_image_offsets: HashMap::default(),
    });

    actor::spawn_player_actor(
        &mut commands,
        &asset_server,
        &mut texture_atlas_layouts,
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    );
    actor::spawn_enemy_actor(
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
    mut query: Query<&mut Transform, With<Camera2d>>,
) {
    if let Ok(mut transform) = query.single_mut() {
        if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
            transform.translation.y += 16.0;
        } else if keyboard_input.pressed(KeyCode::ArrowDown)
            || keyboard_input.pressed(KeyCode::KeyS)
        {
            transform.translation.y -= 16.0;
        } else if keyboard_input.pressed(KeyCode::ArrowLeft)
            || keyboard_input.pressed(KeyCode::KeyA)
        {
            transform.translation.x -= 16.0;
        } else if keyboard_input.pressed(KeyCode::ArrowRight)
            || keyboard_input.pressed(KeyCode::KeyD)
        {
            transform.translation.x += 16.0;
        }
    }
}

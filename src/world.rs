use bevy::prelude::*;
use bevy_ecs_tiled::prelude::TilemapAnchor;
use bevy_ecs_tilemap::TilemapBundle;
use bevy_ecs_tilemap::map::{
    TilemapGridSize, TilemapId, TilemapSize, TilemapSpacing, TilemapTexture, TilemapTileSize,
    TilemapType,
};
use bevy_ecs_tilemap::tiles::{TileBundle, TileFlip, TilePos, TileStorage, TileTextureIndex};

use crate::common::Direction;
use crate::tiled_plugin::TiledMapWithTextures;
use crate::{RandomSource, building, environment};
use rand::Rng;

#[derive(Component, Clone)]
pub enum TileType {
    None,
    EmptyGround(environment::Ground),
    GroundWithBuilding(environment::Ground, building::Building),
}

#[derive(Resource)]
pub struct WorldWithTiles {
    pub width: u32,
    pub height: u32,
    pub loaded: bool,
    pub tiles: Vec<TileType>,
}

impl WorldWithTiles {
    pub fn build_world(
        mut commands: Commands,
        mut world: ResMut<WorldWithTiles>,
        mut rng: ResMut<RandomSource>,
        tiled_map: Res<TiledMapWithTextures>,
    ) {
        if !tiled_map.loaded {
            return;
        }
        if world.loaded {
            return;
        }
        let width = world.width;
        let height = world.height;
        let map = tiled_map.map.as_ref().unwrap();

        let mut need_ground = world.width * world.height;
        for _ in 0..30 {
            let x = rng.0.random_range(0..world.width);
            let y = rng.0.random_range(0..world.height);
            if let TileType::None = world.tiles[(y * world.width + x) as usize] {
                need_ground -= 1;
                world.tiles[(y * width + x) as usize] =
                    TileType::EmptyGround(environment::Ground::random(&mut rng.0));
            }
        }

        while need_ground > 0 {
            for x in 0..world.width {
                for y in 0..world.height {
                    let index = (y * world.width + x) as usize;
                    if let TileType::EmptyGround(ref ground) = world.tiles[index] {
                        let direction = Direction::random(&mut rng.0);
                        let next_y = (y as i32 + direction.y());
                        let next_x = (x as i32 + direction.x());
                        if next_y >= world.height as i32
                            || next_x >= world.width as i32
                            || next_x < 0
                            || next_y < 0
                        {
                            continue;
                        }
                        let next = (next_y as u32 * world.width + next_x as u32) as usize;
                        if let TileType::None = world.tiles[next] {
                            need_ground -= 1;

                            world.tiles[next] = TileType::EmptyGround(ground.clone());
                        }
                    }
                    if need_ground == 0 {
                        break;
                    }
                }
                if need_ground == 0 {
                    break;
                }
            }
        }

        for (tileset_index, _) in map.tilesets().iter().enumerate() {
            let Some(tilemap_texture) = tiled_map.tilemap_textures.get(&tileset_index) else {
                warn!("Skipped creating layer with missing tilemap textures.");
                continue;
            };

            let tile_size = TilemapTileSize { x: 16f32, y: 16f32 };

            let tile_spacing = TilemapSpacing {
                x: 0 as f32,
                y: 0 as f32,
            };

            let map_size = TilemapSize {
                x: world.width,
                y: world.height,
            };

            let grid_size = TilemapGridSize { x: 16f32, y: 16f32 };

            let map_type = TilemapType::Square;

            let mut tile_storage = TileStorage::empty(map_size);
            let layer_entity = commands.spawn_empty().id();

            let layer_index = 0u32;
            for x in 0..world.width {
                for y in 0..world.height {
                    let (tiles_tileset_index, layer_tile_id) = world.tiles
                        [(y * world.width + x) as usize]
                        .get_tileset_index_and_layer_id(layer_index, &mut rng.0);
                    if tiles_tileset_index != tileset_index {
                        continue;
                    }

                    let texture_index = match tilemap_texture {
                                    TilemapTexture::Single(_) => layer_tile_id,
                                    TilemapTexture::Vector(_) =>
                                        *tiled_map.tile_image_offsets.get(&(tileset_index, layer_tile_id))
                                        .expect("The offset into to image vector should have been saved during the initial load."),
                                    _ => unreachable!()
                                };

                    let tile_pos = TilePos { x, y };
                    let tile_entity = commands
                        .spawn(TileBundle {
                            position: tile_pos,
                            tilemap_id: TilemapId(layer_entity),
                            texture_index: TileTextureIndex(texture_index),
                            flip: TileFlip {
                                x: false,
                                y: false,
                                d: false,
                            },
                            ..Default::default()
                        })
                        .id();
                    tile_storage.set(&tile_pos, tile_entity);
                }
            }

            commands.entity(layer_entity).insert(TilemapBundle {
                grid_size,
                size: map_size,
                storage: tile_storage,
                texture: tilemap_texture.clone(),
                tile_size,
                spacing: tile_spacing,
                anchor: TilemapAnchor::Center,
                transform: Transform::from_xyz(0f32, 0f32, -1f32 as f32),
                map_type,
                ..Default::default()
            });
        }

        world.loaded = true;
    }
}

impl TileType {
    pub fn get_tileset_index_and_layer_id(
        &self,
        layer_index: u32,
        rng: &mut impl Rng,
    ) -> (usize, u32) {
        match self {
            TileType::None => panic!("Cannot get tileset index and layer ID for None tile type"),
            TileType::EmptyGround(ground) => (ground.tileset_index(), ground.animation_index(rng)),
            TileType::GroundWithBuilding(ground, building) => {
                match layer_index {
                    0 => (ground.tileset_index(), ground.animation_index(rng)),
                    //1 => (building.tileset_index(), building.layer_id()),
                    _ => panic!("Invalid layer index for TileType::GroundWithBuilding"),
                }
            }
        }
    }
}

use bevy_ecs_tiled::prelude::TilemapAnchor;
use bevy_ecs_tilemap::TilemapBundle;
use bevy_ecs_tilemap::map::{
    TilemapGridSize, TilemapId, TilemapSize, TilemapSpacing, TilemapTexture, TilemapTileSize,
    TilemapType,
};
use bevy_ecs_tilemap::tiles::{TileBundle, TileFlip, TilePos, TileStorage, TileTextureIndex};
use rand::seq::SliceRandom;

use crate::building::resources::Resourceable;
use crate::common::Direction;
use crate::world::goods::{ChunksWithResource, Lumber, Ore};
use crate::world::*;
use crate::{RandomSource, environment};
use rand::RngExt;

pub fn build_world(
    mut commands: Commands,
    mut rng: ResMut<RandomSource>,
    mut world: ResMut<WorldWithTiles>,
    asset_server: Res<AssetServer>,
) {
    let dirt_texture_handle: Handle<Image> = asset_server.load("tinyworld/Ground/DeadGrass.png");
    let grass_texture_handle: Handle<Image> =
        asset_server.load("tinyworld/Ground/TexturedGrass.png");
    let water_texture_handle: Handle<Image> = asset_server.load("tinyworld/Ground/Shore.png");
    let shore_texture_handle: Handle<Image> = asset_server.load("tinyworld/Ground/Cliff-Water.png");

    let (width, height) = (world.width, world.height);

    let mut grounded = Vec::new();
    world.tiles[((height / 2) * width + width / 2) as usize] =
        TileType::EmptyGround(environment::Ground::Grass(1));
    let next_ground = TileType::EmptyGround(environment::Ground::Grass(3));
    grounded.push((width / 2 + 1, height / 2, next_ground.clone()));
    grounded.push((width / 2 - 1, height / 2, next_ground.clone()));
    grounded.push((width / 2, height / 2 + 1, next_ground.clone()));
    grounded.push((width / 2, height / 2 - 1, next_ground.clone()));

    for _ in 0..width {
        let x = rng.0.random_range(0..world.width);
        let y = rng.0.random_range(0..world.height);
        if let TileType::None = world.tiles[(y * world.width + x) as usize] {
            let next_ground = TileType::EmptyGround(environment::Ground::random(&mut rng.0));
            grounded.push((x, y, next_ground));
        }
    }

    while grounded.len() > 0 {
        grounded.shuffle(&mut rng.0);
        let mut new_grounded = Vec::new();
        let upper_bound = (grounded.len() / 4).max(1);
        for (x, y, next_ground) in grounded.drain(..upper_bound) {
            if y >= world.height || x >= world.width {
                continue;
            }
            if let TileType::None = world.tiles[(y * world.width + x) as usize] {
                world.tiles[(y * width + x) as usize] = next_ground.clone();

                let east_x = (x as i32).saturating_add(Direction::East.x());
                let east_y = (y as i32).saturating_add(Direction::East.y());
                new_grounded.push((east_x as u32, east_y as u32, next_ground.clone()));

                let west_x = (x as i32).saturating_add(Direction::West.x());
                let west_y = (y as i32).saturating_add(Direction::West.y());
                new_grounded.push((west_x as u32, west_y as u32, next_ground.clone()));

                let north_x = (x as i32).saturating_add(Direction::North.x());
                let north_y = (y as i32).saturating_add(Direction::North.y());
                new_grounded.push((north_x as u32, north_y as u32, next_ground.clone()));

                let south_x = (x as i32).saturating_add(Direction::South.x());
                let south_y = (y as i32).saturating_add(Direction::South.y());
                new_grounded.push((south_x as u32, south_y as u32, next_ground.clone()));
            }
        }
        grounded.extend(new_grounded);
    }
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
    let mut grass_tile_storage = (commands.spawn_empty().id(), TileStorage::empty(map_size));
    let mut dirt_tile_storage = (commands.spawn_empty().id(), TileStorage::empty(map_size));
    let mut water_tile_storage = (commands.spawn_empty().id(), TileStorage::empty(map_size));
    let mut shore_tile_storage = (commands.spawn_empty().id(), TileStorage::empty(map_size));

    let layer_index = 0u32;
    for x in 0..world.width {
        for y in 0..world.height {
            let (tiles_tileset_index, layer_tile_id) = world.tiles[(y * world.width + x) as usize]
                .get_tileset_index_and_layer_id(layer_index, &mut rng.0);

            let tile_pos = TilePos { x, y };
            let layer_storage = match tiles_tileset_index {
                0 => &mut dirt_tile_storage,
                1 => &mut grass_tile_storage,
                2 => &mut shore_tile_storage,
                3 => &mut water_tile_storage,
                _ => unreachable!("Invalid tileset index"),
            };
            let tile_entity = commands
                .spawn((
                    TileBundle {
                        position: tile_pos,
                        tilemap_id: TilemapId(layer_storage.0),
                        texture_index: TileTextureIndex(layer_tile_id),
                        flip: TileFlip {
                            x: false,
                            y: false,
                            d: false,
                        },
                        ..Default::default()
                    },
                    IsTile,
                ))
                .id();
            layer_storage.1.set(&tile_pos, tile_entity);
        }
    }
    commands.entity(dirt_tile_storage.0).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: dirt_tile_storage.1,
        texture: TilemapTexture::Single(dirt_texture_handle),
        tile_size,
        spacing: tile_spacing,
        anchor: TilemapAnchor::Center,
        transform: Transform::from_xyz(0f32, 0f32, -1f32 as f32),
        map_type,
        ..Default::default()
    });
    commands.entity(grass_tile_storage.0).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: grass_tile_storage.1,
        texture: TilemapTexture::Single(grass_texture_handle),
        tile_size,
        spacing: tile_spacing,
        anchor: TilemapAnchor::Center,
        transform: Transform::from_xyz(0f32, 0f32, -1f32 as f32),
        map_type,
        ..Default::default()
    });
    commands.entity(shore_tile_storage.0).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: shore_tile_storage.1,
        texture: TilemapTexture::Single(shore_texture_handle),
        tile_size,
        spacing: tile_spacing,
        anchor: TilemapAnchor::Center,
        transform: Transform::from_xyz(0f32, 0f32, -1f32 as f32),
        map_type,
        ..Default::default()
    });
    commands.entity(water_tile_storage.0).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: water_tile_storage.1,
        texture: TilemapTexture::Single(water_texture_handle),
        tile_size,
        spacing: tile_spacing,
        anchor: TilemapAnchor::Center,
        transform: Transform::from_xyz(0f32, 0f32, -1f32 as f32),
        map_type,
        ..Default::default()
    });

    world.map_size = map_size;
    world.grid_size = grid_size;
    world.tile_size = tile_size;
    world.tilemap_type = map_type;
    world.tilemap_anchor = TilemapAnchor::Center;
    world.map_type = map_type;
    world.transform = Transform::from_xyz(0f32, 0f32, 0f32);
}

pub fn spawn_castle(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut rng: ResMut<RandomSource>,
    mut world: ResMut<WorldWithTiles>,
) {
    let entity = building::Building::Castle.spawn(
        &mut commands,
        &asset_server,
        &mut texture_atlas_layouts,
        &mut rng.0,
        Vec3::new(0.0, 0.0, 0.0),
    );
    let tile_pos = TilePos {
        x: (world.width / 2) as u32 - 1,
        y: (world.height / 2) as u32 - 1,
    };
    let width = world.width;
    let previous_tile = world.tiles[(tile_pos.y * width + tile_pos.x) as usize].clone();
    world.tiles[(tile_pos.y * width + tile_pos.x) as usize] =
        previous_tile.insert_building_part(entity, BuildingPart::TLCastle);
    let previous_tile = world.tiles[(tile_pos.y * width + tile_pos.x + 1) as usize].clone();
    world.tiles[(tile_pos.y * width + tile_pos.x + 1) as usize] =
        previous_tile.insert_building_part(entity, BuildingPart::TRCastle);
    let previous_tile = world.tiles[(tile_pos.y * width + tile_pos.x + width) as usize].clone();
    world.tiles[(tile_pos.y * width + tile_pos.x + width) as usize] =
        previous_tile.insert_building_part(entity, BuildingPart::BLCastle);
    let previous_tile = world.tiles[(tile_pos.y * width + tile_pos.x + width + 1) as usize].clone();
    world.tiles[(tile_pos.y * width + tile_pos.x + width + 1) as usize] =
        previous_tile.insert_building_part(entity, BuildingPart::BRCastle);
}

#[derive(Message)]
pub struct ChangeTileComposition(pub u32, pub u32);

impl WorldWithTiles {
    pub fn insert_building(
        &mut self,
        position: Vec3,
        building: Building,
        entity: Entity,
        change_registry: &mut MessageWriter<ChangeTileComposition>,
    ) {
        let tile_pos = TilePos::from_world_pos(
            &Vec2::new(position.x, position.y),
            &self.map_size,
            &self.grid_size,
            &self.tile_size,
            &self.tilemap_type,
            &self.tilemap_anchor,
        );
        if let Some(tile_pos) = tile_pos {
            let width = self.width;
            let previous_tile = self.tiles[(tile_pos.y * width + tile_pos.x) as usize].clone();
            self.tiles[(tile_pos.y * width + tile_pos.x) as usize] =
                previous_tile.insert_building(entity, building);
            change_registry.write(ChangeTileComposition(tile_pos.x, tile_pos.y));
        }
    }

    pub fn insert_construction(
        &mut self,
        tile_pos: TilePos,
        construction: Construction,
        change_registry: &mut MessageWriter<ChangeTileComposition>,
    ) {
        let width = self.width;
        let previous_tile = self.tiles[(tile_pos.y * width + tile_pos.x) as usize].clone();
        self.tiles[(tile_pos.y * width + tile_pos.x) as usize] =
            previous_tile.insert_construction(construction);
        change_registry.write(ChangeTileComposition(tile_pos.x, tile_pos.y));
    }

    pub fn insert_resource<T: Resourceable>(
        &mut self,
        tile_pos: TilePos,
        entity: Entity,
        change_registry: &mut MessageWriter<ChangeTileComposition>,
        chunks: &mut ChunksWithResource<T>,
    ) {
        let width = self.width;
        let previous_tile = self.tiles[(tile_pos.y * width + tile_pos.x) as usize].clone();
        self.tiles[(tile_pos.y * width + tile_pos.x) as usize] =
            previous_tile.insert_resource::<T>(entity);
        change_registry.write(ChangeTileComposition(tile_pos.x, tile_pos.y));
        chunks.add(tile_pos.x, tile_pos.y);
    }

    pub fn remove_resource<T: Resourceable>(
        &mut self,
        tile_pos: TilePos,
        change_registry: &mut MessageWriter<ChangeTileComposition>,
        chunks: &mut ChunksWithResource<T>,
    ) {
        let width = self.width;
        let previous_tile = self.tiles[(tile_pos.y * width + tile_pos.x) as usize].clone();
        self.tiles[(tile_pos.y * width + tile_pos.x) as usize] = previous_tile.remove_structure();
        change_registry.write(ChangeTileComposition(tile_pos.x, tile_pos.y));
        chunks.remove(tile_pos.x, tile_pos.y);
    }

    pub fn is_passable(&self, position: TilePos) -> bool {
        self.tiles[(position.y * self.width + position.x) as usize].is_passable()
    }

    pub fn can_place_building(&self, position: TilePos) -> bool {
        let width = self.width;
        let height = self.height;
        if !self.tiles[(position.y * width + position.x) as usize].can_place_building() {
            return false;
        }
        for x in -1i32..2i32 {
            for y in -1i32..2i32 {
                if x == 0 && y == 0 {
                    continue;
                }
                let trial_x = position.x as i32 + x;
                let trial_y = position.y as i32 + y;
                if trial_x >= width as i32 || trial_y >= height as i32 || trial_x < 0 || trial_y < 0
                {
                    continue;
                }
                if !self.tiles[(trial_y as u32 * width + trial_x as u32) as usize].is_passable() {
                    return false;
                }
            }
        }
        return true;
    }

    pub fn can_place_lumber(&self, position: TilePos) -> bool {
        let width = self.width;
        let height = self.height;
        if !self.tiles[(position.y * width + position.x) as usize].can_place_lumber() {
            return false;
        }
        let mut number_of_passable = 0u32;
        for x in -1i32..2i32 {
            for y in -1i32..2i32 {
                if x == 0 && y == 0 {
                    continue;
                }
                let trial_x = position.x as i32 + x;
                let trial_y = position.y as i32 + y;
                if trial_x >= width as i32 || trial_y >= height as i32 || trial_x < 0 || trial_y < 0
                {
                    return false;
                }
                if !self.tiles[(trial_y as u32 * width + trial_x as u32) as usize].is_passable() {
                    number_of_passable += 1;
                }
            }
        }
        return number_of_passable < 6;
    }

    pub fn can_place_deposit(&self, position: TilePos) -> bool {
        let width = self.width;
        if !self.tiles[(position.y * width + position.x) as usize].can_place_deposit() {
            return false;
        }
        let height = self.height;
        for x in -1i32..2i32 {
            for y in -1i32..2i32 {
                if x == 0 && y == 0 {
                    continue;
                }
                let trial_x = position.x as i32 + x;
                let trial_y = position.y as i32 + y;
                if trial_x >= width as i32 || trial_y >= height as i32 || trial_x < 0 || trial_y < 0
                {
                    continue;
                }
                if !self.tiles[(trial_y as u32 * width + trial_x as u32) as usize]
                    .can_place_deposit()
                {
                    return false;
                }
            }
        }
        return true;
    }
}

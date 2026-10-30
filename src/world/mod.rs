use bevy::prelude::*;
use bevy_ecs_tiled::prelude::TilemapAnchor;
use bevy_ecs_tilemap::map::{TilemapGridSize, TilemapSize, TilemapTileSize, TilemapType};
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_northstar::nav::Nav;
use saveable::Saveable;

use crate::building::Building;
use crate::building::construction::Construction;
use crate::building::resources::{ResourceKind, Resourceable};
use crate::{building, environment};
use rand::Rng;

pub mod goods;
pub mod hover_selector;
pub mod pathfinding;
pub mod plugin;
pub mod tiles;

/*
TODO: CHUNKING
MAYBE RIVERS
 */

#[derive(Component, Clone, Saveable)]
pub enum BuildingPart {
    TLCastle,
    TRCastle,
    BLCastle,
    BRCastle,
}
#[derive(Component, Clone, Saveable)]
pub enum Structure {
    Building(Entity, Building),
    Construction(Construction),
    BuildingPart(Entity, BuildingPart),
    Resource(ResourceKind, Entity),
}

#[derive(Component, Clone, Saveable)]
pub enum TileType {
    None,
    EmptyGround(environment::Ground),
    GroundWithBuilding(environment::Ground, Structure),
}

#[derive(Resource)]
pub struct WorldWithTiles {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<TileType>,

    pub map_size: TilemapSize,
    pub grid_size: TilemapGridSize,
    pub tile_size: TilemapTileSize,
    pub tilemap_type: TilemapType,
    pub tilemap_anchor: TilemapAnchor,
    pub map_type: TilemapType,
    pub transform: Transform,
}

#[derive(Component)]
pub struct IsTile;

impl TileType {
    pub fn get_tileset_index_and_layer_id(
        &self,
        layer_index: u32,
        rng: &mut impl Rng,
    ) -> (usize, u32) {
        match self {
            TileType::None => panic!("Cannot get tileset index and layer ID for None tile type"),
            TileType::EmptyGround(ground) => (ground.tileset_index(), ground.animation_index(rng)),
            TileType::GroundWithBuilding(ground, _) => {
                match layer_index {
                    0 => (ground.tileset_index(), ground.animation_index(rng)),
                    //1 => (building.tileset_index(), building.layer_id()),
                    _ => panic!("Invalid layer index for TileType::GroundWithBuilding"),
                }
            }
        }
    }
}

impl TileType {
    pub fn is_passable(&self) -> bool {
        match self {
            TileType::None => false,
            TileType::EmptyGround(ground) => ground.is_passable(),
            TileType::GroundWithBuilding(ground, building) => {
                ground.is_passable() && building.is_passable()
            }
        }
    }

    pub fn can_place_building(&self) -> bool {
        match self {
            TileType::None => false,
            TileType::EmptyGround(ground) => ground.can_place_building(),
            TileType::GroundWithBuilding(ground, building) => {
                ground.can_place_building() && building.can_place_building()
            }
        }
    }

    pub fn can_place_lumber(&self) -> bool {
        match self {
            TileType::None => false,
            TileType::EmptyGround(ground) => ground.can_place_lumber(),
            TileType::GroundWithBuilding(ground, building) => {
                ground.can_place_lumber() && building.can_place_building()
            }
        }
    }

    pub fn can_place_deposit(&self) -> bool {
        match self {
            TileType::None => false,
            TileType::EmptyGround(ground) => ground.can_place_ore(),
            TileType::GroundWithBuilding(ground, building) => {
                ground.can_place_ore() && building.can_place_building()
            }
        }
    }

    pub fn nav(&self) -> Nav {
        match self {
            TileType::None => Nav::Impassable,
            TileType::EmptyGround(ground) => ground.nav(),
            TileType::GroundWithBuilding(ground, building) => {
                if building.is_passable() {
                    return ground.nav();
                }
                building.nav()
            }
        }
    }

    pub fn insert_building(self, entity: Entity, building: Building) -> Self {
        match self {
            TileType::None => panic!("Cannot insert building into None tile type"),
            TileType::EmptyGround(ground) => {
                TileType::GroundWithBuilding(ground, Structure::Building(entity, building))
            }
            TileType::GroundWithBuilding(ground, Structure::Construction(_)) => {
                TileType::GroundWithBuilding(ground, Structure::Building(entity, building))
            }
            TileType::GroundWithBuilding(_, _) => {
                panic!("Cannot insert building into a tile that already has a building")
            }
        }
    }

    pub fn insert_construction(self, construction: Construction) -> Self {
        match self {
            TileType::None => panic!("Cannot insert building into None tile type"),
            TileType::EmptyGround(ground) => {
                TileType::GroundWithBuilding(ground, Structure::Construction(construction))
            }
            TileType::GroundWithBuilding(_, _) => {
                panic!("Cannot insert building into a tile that already has a building")
            }
        }
    }

    pub fn insert_resource<T: Resourceable>(self, entity: Entity) -> Self {
        let kind = T::kind();
        match self {
            TileType::None => panic!("Cannot insert building into None tile type"),
            TileType::EmptyGround(ground) => {
                TileType::GroundWithBuilding(ground, Structure::Resource(kind, entity))
            }
            TileType::GroundWithBuilding(_, _) => {
                panic!("Cannot insert building into a tile that already has a building")
            }
        }
    }

    pub fn insert_building_part(self, entity: Entity, building_part: BuildingPart) -> Self {
        match self {
            TileType::None => panic!("Cannot insert building part into None tile type"),
            TileType::EmptyGround(ground) => {
                TileType::GroundWithBuilding(ground, Structure::BuildingPart(entity, building_part))
            }
            TileType::GroundWithBuilding(_, _) => {
                panic!("Cannot insert building part into a tile that already has a building")
            }
        }
    }

    pub fn remove_structure(self) -> Self {
        match self {
            TileType::None => panic!("Cannot remove building part into from tile type"),
            TileType::EmptyGround(ground) => {
                warn!("Why are we removing when there's nothing there to remove?!");
                TileType::EmptyGround(ground)
            }
            TileType::GroundWithBuilding(ground, _) => TileType::EmptyGround(ground),
        }
    }

    pub fn get_building_entity(&self) -> Option<(Entity, Building)> {
        match self {
            TileType::None => None,
            TileType::EmptyGround(_) => None,
            TileType::GroundWithBuilding(_, structure) => match structure {
                Structure::Building(entity, building) => Some((*entity, building.clone())),
                Structure::Construction(_) => None,
                Structure::BuildingPart(entity, part) => Some((
                    *entity,
                    match part {
                        BuildingPart::TLCastle => Building::Castle,
                        BuildingPart::TRCastle => Building::Castle,
                        BuildingPart::BLCastle => Building::Castle,
                        BuildingPart::BRCastle => Building::Castle,
                    },
                )),
                Structure::Resource(_, _) => None,
            },
        }
    }
}

impl WorldWithTiles {
    pub fn get_position(&self, transform: &Transform) -> Option<TilePos> {
        return TilePos::from_world_pos(
            &transform.translation.xy(),
            &self.map_size,
            &self.grid_size,
            &self.tile_size,
            &self.map_type,
            &self.tilemap_anchor,
        );
    }

    pub fn get_xy(&self, xy: Vec2) -> Option<TilePos> {
        return TilePos::from_world_pos(
            &xy,
            &self.map_size,
            &self.grid_size,
            &self.tile_size,
            &self.map_type,
            &self.tilemap_anchor,
        );
    }

    pub fn get_transform(&self, tile_pos: TilePos) -> Vec3 {
        let center = tile_pos.center_in_world(
            &self.map_size,
            &self.grid_size,
            &self.tile_size,
            &self.map_type,
            &self.tilemap_anchor,
        );
        Vec3::new(center.x, center.y, 0.0)
    }

    pub fn get_resource(&self, kind: &ResourceKind, x: u32, y: u32) -> Option<Entity> {
        if x >= self.width || y >= self.height {
            return None;
        }
        match &self.tiles[(y * self.width + x) as usize] {
            TileType::GroundWithBuilding(_, Structure::Resource(resource_kind, entity))
                if *resource_kind == *kind =>
            {
                Some(*entity)
            }
            _ => None,
        }
    }
}

impl Structure {
    pub fn is_passable(&self) -> bool {
        match self {
            Structure::Building(_, building) => building.is_passable(),
            Structure::BuildingPart(_, _) => false,
            Structure::Construction(_) => false,
            Structure::Resource(_, _) => false,
        }
    }
    pub fn nav(&self) -> Nav {
        match self {
            Structure::Building(_, building) => {
                if building.is_passable() {
                    return Nav::Passable(5);
                }
                Nav::Impassable
            }
            Structure::BuildingPart(_, _) => Nav::Impassable,
            Structure::Construction(_) => Nav::Passable(5),
            Structure::Resource(_, _) => Nav::Impassable,
        }
    }
    pub fn can_place_building(&self) -> bool {
        match self {
            Structure::Building(_, building) => building.is_passable(),
            Structure::BuildingPart(_, _) => false,
            Structure::Construction(_) => false,
            Structure::Resource(_, _) => false,
        }
    }
}

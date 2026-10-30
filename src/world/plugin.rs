use bevy_ecs_tiled::prelude::TilemapAnchor;

use crate::world::{goods::GoodsPlugin, pathfinding::PathfindingPlugin, *};
use bevy_ecs_tiled::prelude::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (
                setup,
                tiles::build_world,
                tiles::spawn_castle,
                pathfinding::startup.after(tiles::build_world),
                pathfinding::build_grid.after(pathfinding::startup),
            )
                .chain(),
        )
        .add_plugins(GoodsPlugin)
        .add_plugins(PathfindingPlugin);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(TilemapAnchor::default());
    commands.spawn(TilemapRenderSettings::default());
    commands.spawn(TiledMapLayerZOffset::default());

    hover_selector::spawn_hover_selector(&mut commands, &asset_server, &mut texture_atlas_layouts);

    const MAP_SIZE: usize = 256;
    commands.insert_resource(WorldWithTiles {
        width: MAP_SIZE as u32,
        height: MAP_SIZE as u32,
        tiles: vec![TileType::None; MAP_SIZE * MAP_SIZE],
        map_size: TilemapSize::new(MAP_SIZE as u32, MAP_SIZE as u32),
        grid_size: TilemapGridSize::default(),
        tile_size: TilemapTileSize::default(),
        tilemap_type: TilemapType::Square,
        tilemap_anchor: TilemapAnchor::Center,
        map_type: TilemapType::Square,
        transform: Transform::default(),
    });
}

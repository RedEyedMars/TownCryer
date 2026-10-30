use bevy::prelude::*;
use bevy_northstar::prelude::*;

use crate::world::{WorldWithTiles, tiles::ChangeTileComposition};

pub struct PathfindingPlugin;
impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(NorthstarPlugin::<OrdinalNeighborhood>::default())
            .add_message::<ChangeTileComposition>()
            .add_systems(Update, update_grid);
    }
}

pub fn startup(mut commands: Commands, world: Res<WorldWithTiles>) {
    let grid_settings = GridSettingsBuilder::new_2d(world.width, world.height)
        .chunk_size(16)
        .build();

    commands.spawn(OrdinalGrid::new(&grid_settings));
}

pub fn build_grid(grid: Single<&mut Grid<OrdinalNeighborhood>>, world: Res<WorldWithTiles>) {
    let mut grid = grid.into_inner();

    let (width, height) = (world.width, world.height);
    for x in 0..width {
        for y in 0..height {
            grid.set_nav(
                UVec3::new(x, y, 0),
                world.tiles[(y * width + x) as usize].nav(),
            );
        }
    }

    info!("Building the grid...");
    grid.build();
    info!("Grid built successfully!");
}

pub fn update_grid(
    mut reader: MessageReader<ChangeTileComposition>,
    mut grid: Single<&mut OrdinalGrid>,
    world: Res<WorldWithTiles>,
    mut commands: Commands,
    agents: Query<(Entity, &Pathfind)>,
) {
    let mut has_any = false;
    let width = world.width;
    for tile in reader.read() {
        has_any = true;
        let (x, y) = (tile.0, tile.1);
        grid.set_nav(
            UVec3::new(x, y, 0),
            world.tiles[(y * width + x) as usize].nav(),
        );
    }
    if !has_any {
        return;
    }

    info!("Building the grid...");
    grid.build();
    info!("Grid built successfully!");

    for (entity, pathfind) in &agents {
        commands
            .entity(entity)
            .remove::<(NextPos, Path)>()
            .insert(Pathfind::new(pathfind.goal));
    }
}

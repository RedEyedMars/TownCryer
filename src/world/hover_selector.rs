use bevy_ecs_tilemap::tiles::TilePos;

use crate::building::Building;
use crate::world::*;

#[derive(Component)]
pub enum HoverState {
    Neutral,
    Inspecting,
    CreatingBuilding,
}
#[derive(Component)]
pub struct HoverSelector {
    pub tile_pos: Option<TilePos>,
    pub building: Option<Building>,
}

#[derive(Component)]
pub struct HoverSelectorTimer {
    pub timer: Timer,
}

pub fn spawn_hover_selector(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("tinyworld/User Interface/BoxSelector.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 2, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    commands.spawn((
        HoverSelector {
            tile_pos: None,
            building: None,
        },
        HoverState::Inspecting,
        HoverSelectorTimer {
            timer: Timer::from_seconds(0.1, TimerMode::Once),
        },
        Sprite::from_atlas_image(
            texture,
            TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            },
        ),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        //     Visibility::Hidden,
        Pickable::IGNORE,
        Name::new("Hover Selector"),
    ));
}

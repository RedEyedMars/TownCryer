use bevy::prelude::*;

use crate::{
    building::Building,
};

#[derive(Component)]
pub struct Castle;

#[derive(Component)]
pub enum CastleAction {
    Construct(Building),
}

#[derive(Component)]
pub struct CastleState {
    pub action: Option<CastleAction>,
}

#[derive(Component)]
pub struct CastleTimer {
    pub timer: Timer,
}

impl Castle {
    pub fn spawn(
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        position: Vec3,
    ) {
        let texture = asset_server.load("tinyworld/Buildings/Purple/PurpleKeep.png");
        let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), 3, 2, None, None);
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let mut transform = Transform::from_translation(position);
        transform.scale = Vec3::splat(2.0);
        commands.spawn((
            Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index: 1,
                },
            ),
            transform,
            Castle,
            CastleState { action: None },
            CastleTimer {
                timer: Timer::from_seconds(5.0, TimerMode::Once),
            },
        ));
    }
}
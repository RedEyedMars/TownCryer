use bevy::prelude::*;

use rand::Rng;

#[derive(Component, Clone)]
pub enum Building {
    House,
    Farm,
    Barracks,
    Market,
}

impl Building {
    pub fn sprite_path(&self) -> &str {
        match self {
            Building::House => "tinyworld/Buildings/Purple/PurpleHouses.png",
            Building::Farm => "tinyworld/Buildings/Purple/PurpleResources.png",
            Building::Barracks => "tinyworld/Buildings/Purple/PurpleBarracks.png",
            Building::Market => "tinyworld/Buildings/Purple/PurpleMarket.png",
        }
    }

    pub fn gen_sprite_index(&self, rng: &mut impl Rng) -> usize {
        match self {
            Building::House => rng.gen_range(0..12),
            Building::Farm => rng.gen_range(0..3),
            Building::Barracks => 0,
            Building::Market => rng.gen_range(0..12),
        }
    }

    pub fn sprite_dimensions(&self) -> (u8, u8) {
        match self {
            Building::House => (3, 4),
            Building::Farm => (3, 5),
            Building::Barracks => (4, 5),
            Building::Market => (3, 4),
        }
    }

    pub fn random(rng: &mut impl Rng) -> Self {
        match rng.gen_range(0..4) {
            0 => Building::House,
            1 => Building::Farm,
            2 => Building::Barracks,
            _ => Building::Market,
        }
    }

    pub fn spawn(
        &self,
        commands: &mut Commands,
        asset_server: &Res<AssetServer>,
        texture_atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
        rng: &mut impl Rng,
        position: Vec3,
    ) {
        let texture = asset_server.load(self.sprite_path());
        let index = self.gen_sprite_index(rng);
        let (width, height) = self.sprite_dimensions();

        let layout = TextureAtlasLayout::from_grid(
            UVec2::splat(16),
            width as u32,
            height as u32,
            None,
            None,
        );
        let texture_atlas_layout = texture_atlas_layouts.add(layout);

        let mut transform = Transform::from_translation(position);
        transform.scale = Vec3::splat(2.0);
        commands.spawn((
            Sprite::from_atlas_image(
                texture,
                TextureAtlas {
                    layout: texture_atlas_layout,
                    index,
                },
            ),
            transform,
            self.clone(),
        ));
    }
}

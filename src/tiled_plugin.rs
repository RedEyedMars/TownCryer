// How to use this:
//   You should copy/paste this into your project and use it much like examples/tiles.rs uses this
//   file. When you do so you will need to adjust the code based on whether you're using the
//   'atlas` feature in bevy_ecs_tilemap. The bevy_ecs_tilemap uses this as an example of how to
//   use both single image tilesets and image collection tilesets. Since your project won't have
//   the 'atlas' feature defined in your Cargo config, the expressions prefixed by the #[cfg(...)]
//   macro will not compile in your project as-is. If your project depends on the bevy_ecs_tilemap
//   'atlas' feature then move all of the expressions prefixed by #[cfg(not(feature = "atlas"))].
//   Otherwise remove all of the expressions prefixed by #[cfg(feature = "atlas")].
//
// Functional limitations:
//   * When the 'atlas' feature is enabled tilesets using a collection of images will be skipped.
//   * Only finite tile layers are loaded. Infinite tile layers and object layers will be skipped.


use bevy::asset::AssetServer;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::ResMut;
use bevy::log::info;
use bevy::{
    asset::{AssetLoader, AssetPath},
    platform::collections::HashMap,
    prelude::{
        Assets, Handle, Image, Res,
    },
    reflect::TypePath,
};
use bevy_ecs_tiled::prelude::TiledMap;
use bevy_ecs_tilemap::prelude::*;

#[derive(TypePath, Resource)]
pub struct TiledMapWithTextures {
    pub loaded: bool,
    pub handle: Handle<TiledMap>,
    pub map: Option<tiled::Map>,
    pub tilemap_textures: HashMap<usize, TilemapTexture>,
    pub tile_image_offsets: HashMap<(usize, tiled::TileId), u32>,
}
pub fn load(
    atlases: Res<Assets<TiledMap>>,
    asset_server: Res<AssetServer>,
    mut asset_map: ResMut<TiledMapWithTextures>,
) {
    if asset_map.loaded {
        return;
    }
    let map_handle = asset_map.handle.id();
    let maybe_load_context = atlases.get(map_handle);
    if maybe_load_context.is_none() {
        return;
    }
    let load_context = maybe_load_context.expect("TiledMapHandle should have a valid TiledMap");
    let map = &load_context.map;

    let mut tilemap_textures = HashMap::default();
    let mut tile_image_offsets = HashMap::default();

    for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
        let tilemap_texture = match &tileset.image {
            None => {
                let mut tile_images: Vec<Handle<Image>> = Vec::new();
                for (tile_id, tile) in tileset.tiles() {
                    if let Some(img) = &tile.image {
                        // The load context path is the TMX file itself. If the file is at the root of the
                        // assets/ directory structure then the tmx_dir will be empty, which is fine.
                        let tmx_dir = load_context
                            .map
                            .source
                            .parent()
                            .expect("The asset load context was empty.");
                        let tile_path = tmx_dir.join(&img.source);
                        let asset_path = AssetPath::from(tile_path);
                        info!(
                            "Loading tile image from {asset_path:?} as image ({tileset_index}, {tile_id})"
                        );
                        let texture: Handle<Image> = asset_server.load(asset_path.clone());
                        tile_image_offsets
                            .insert((tileset_index, tile_id), tile_images.len() as u32);
                        tile_images.push(texture.clone());
                    }
                }

                TilemapTexture::Vector(tile_images)
            }
            Some(img) => {
                // The load context path is the TMX file itself. If the file is at the root of the
                // assets/ directory structure then the tmx_dir will be empty, which is fine.
                let tmx_dir = load_context
                    .map
                    .source
                    .parent()
                    .expect("The asset load context was empty.");
                let tile_path = tmx_dir.join(&img.source);
                let asset_path = AssetPath::from(tile_path);
                let texture: Handle<Image> = asset_server.load(asset_path.clone());

                TilemapTexture::Single(texture.clone())
            }
        };

        tilemap_textures.insert(tileset_index, tilemap_texture);
    }

    info!("Loaded map: {}", load_context.map.source.display());
    asset_map.loaded = true;
    asset_map.map = Some(load_context.map.clone());
    asset_map.tilemap_textures = tilemap_textures;
    asset_map.tile_image_offsets = tile_image_offsets;
}

use std::mem::replace;

use anyhow::{Error, Result};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    render::texture::{ImageLoader, ImageLoaderSettings},
};
use serde::{Deserialize, Serialize};

use crate::{
    animation::AnimationAsset,
    image::PxImage,
    palette::asset_palette,
    position::{PxLayer, Spatial},
    prelude::*,
};

pub(crate) fn plug(app: &mut App) {
    app.init_asset::<PxTileset>()
        .init_asset_loader::<PxTilesetLoader>();
}

#[derive(Serialize, Deserialize)]
struct PxTilesetLoaderSettings {
    tile_size: UVec2,
    image_loader_settings: ImageLoaderSettings,
}

impl Default for PxTilesetLoaderSettings {
    fn default() -> Self {
        Self {
            tile_size: UVec2::ONE,
            image_loader_settings: default(),
        }
    }
}

struct PxTilesetLoader(ImageLoader);

impl FromWorld for PxTilesetLoader {
    fn from_world(world: &mut World) -> Self {
        Self(ImageLoader::from_world(world))
    }
}

impl AssetLoader for PxTilesetLoader {
    type Asset = PxTileset;
    type Settings = PxTilesetLoaderSettings;
    type Error = Error;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        settings: &'a PxTilesetLoaderSettings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<PxTileset> {
        let Self(image_loader) = self;
        let image = image_loader
            .load(reader, &settings.image_loader_settings, load_context)
            .await?;
        let palette = asset_palette().await;
        let indices = PxImage::palette_indices(palette, &image)?;
        let tile_size = settings.tile_size;
        let tile_area = tile_size.x * tile_size.y;
        let mut tileset = Vec::default();
        let mut tile = Vec::with_capacity(tile_area as usize);
        let tileset_width = image.texture_descriptor.size.width;
        let tile_tileset_width = tileset_width / tile_size.x;
        let mut max_frame_count = 0;

        for i in 0..indices.area() {
            let tile_index = i as u32 / tile_area;
            let tile_pos = i as u32 % tile_area;

            tile.push(
                indices.pixel(
                    (UVec2::new(
                        tile_index % tile_tileset_width,
                        tile_index / tile_tileset_width,
                    ) * tile_size
                        + UVec2::new(tile_pos % tile_size.x, tile_pos / tile_size.y))
                    .as_ivec2(),
                ),
            );

            if tile_pos == tile_area - 1
                && tile_index % tile_tileset_width == tile_tileset_width - 1
            {
                while tile.len() > tile_area as usize
                    && tile[tile.len() - tile_area as usize..tile.len()]
                        .iter()
                        .all(|pixel| pixel.is_none())
                {
                    tile.truncate(tile.len() - tile_area as usize);
                }

                let frame_count = tile.len() / tile_area as usize;
                if max_frame_count < frame_count {
                    max_frame_count = frame_count;
                }

                tileset.push(PxSprite {
                    data: PxImage::new(
                        replace(&mut tile, Vec::with_capacity(tile_area as usize)),
                        tile_size.x as usize,
                    ),
                    frame_size: tile_area as usize,
                });
            }
        }

        Ok(PxTileset {
            tileset,
            tile_size,
            max_frame_count,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["px_tileset.png"]
    }
}

/// A tileset for a tilemap. Create a [`Handle<PxTileset>`] with a [`PxAssets<PxTileset>`]
/// and an image file. The image file contains a column of tiles, ordered from bottom to top.
/// For animated tilesets, add additional frames to the right of tiles, marking the end
/// of an animation with a fully transparent tile or the end of the image.
/// See `assets/tileset/tileset.png` for an example.
#[derive(Asset, Reflect, Debug)]
pub struct PxTileset {
    pub(crate) tileset: Vec<PxSprite>,
    tile_size: UVec2,
    max_frame_count: usize,
}

impl AnimationAsset for PxTileset {
    fn max_frame_count(&self) -> usize {
        self.max_frame_count
    }
}

impl PxTileset {
    /// The size of tiles in the tileset
    pub fn tile_size(&self) -> UVec2 {
        self.tile_size
    }
}

/// A tilemap
#[derive(Component, Clone, Default, Debug)]
pub struct PxMap {
    tiles: Vec<Option<Entity>>,
    width: usize,
}

impl PxMap {
    /// Creates a [`PxMap`]
    pub fn new(size: UVec2) -> Self {
        Self {
            tiles: vec![None; (size.x * size.y) as usize],
            width: size.x as usize,
        }
    }

    fn index(&self, at: UVec2) -> Option<usize> {
        let x = at.x as usize;
        if x >= self.width {
            return None;
        }

        Some(x + at.y as usize * self.width)
    }

    /// Gets a tile. Returns `None` if there is no tile at the given position or if the position is
    /// out of bounds.
    pub fn get(&self, at: UVec2) -> Option<Entity> {
        self.tiles.get(self.index(at)?).copied()?
    }

    /// Gets a tile mutably. Returns `Some(&mut None)` if there is no tile at the given position.
    /// Returns `None` if the position is out of bounds.
    pub fn get_mut(&mut self, at: UVec2) -> Option<&mut Option<Entity>> {
        let index = self.index(at);
        self.tiles.get_mut(index?)
    }

    /// Sets a tile and returns the previous tile at the position. If there was no tile, returns
    /// `None`. If the position is out of bounds, returns `None` and there is no effect.
    pub fn set(&mut self, tile: Option<Entity>, at: UVec2) -> Option<Entity> {
        let target = self.get_mut(at)?;
        let old = *target;
        *target = tile;
        old
    }

    /// Gets the size of the map
    pub fn size(&self) -> UVec2 {
        let width = self.width as u32;
        UVec2::new(width, self.tiles.len() as u32 / width)
    }
}

impl<'a> Spatial for (&'a PxMap, &'a PxTileset) {
    fn frame_size(&self) -> UVec2 {
        let (map, tileset) = self;
        map.size() * tileset.tile_size
    }
}

/// Creates a tilemap
#[derive(Bundle, Debug, Default)]
pub struct PxMapBundle<L: PxLayer> {
    /// A [`PxMap`] component
    pub map: PxMap,
    /// A [`Handle<PxTileset>`] component
    pub tileset: Handle<PxTileset>,
    /// A [`PxPosition`] component
    pub position: PxPosition,
    /// A layer component
    pub layer: L,
    /// A [`PxCanvas`] component
    pub canvas: PxCanvas,
    /// A [`Visibility`] component
    pub visibility: Visibility,
}

/// A tile. Must be added to tiles added to [`PxMap`].
#[derive(Component, Default, Debug)]
pub struct PxTile {
    /// The index to the tile texture in the tileset
    pub texture: u32,
}

impl From<u32> for PxTile {
    fn from(value: u32) -> Self {
        Self { texture: value }
    }
}

/// Creates a tile
#[derive(Bundle, Debug, Default)]
pub struct PxTileBundle {
    /// A [`PxTile`] component
    pub tile: PxTile,
    /// A [`Visibility`] component
    pub visibility: Visibility,
}

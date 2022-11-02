use std::mem::replace;

use crate::{
    animation::AnimationAsset,
    asset::{PxAsset, PxAssetData},
    image::PxImage,
    palette::Palette,
    position::PxLayer,
    prelude::*,
    sprite::PxSpriteData,
};

#[derive(Debug)]
pub struct PxTilesetData {
    pub(crate) tileset: Vec<PxSpriteData>,
    tile_size: UVec2,
    max_frame_count: usize,
}

impl PxAssetData for PxTilesetData {
    const UUID: [u8; 16] = [
        162, 43, 205, 75, 105, 6, 38, 153, 40, 191, 54, 134, 163, 159, 204, 10,
    ];
    type Config = UVec2;

    fn new(palette: &Palette, image: &Image, tile_size: &Self::Config) -> Self {
        let indices = PxImage::palette_indices(palette, image);
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
                    ) * *tile_size
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
                        .all(|pixel| *pixel == None)
                {
                    tile.truncate(tile.len() - tile_area as usize);
                }

                let frame_count = tile.len() / tile_area as usize;
                if max_frame_count < frame_count {
                    max_frame_count = frame_count;
                }

                tileset.push(PxSpriteData {
                    data: PxImage::new(
                        replace(&mut tile, Vec::with_capacity(tile_area as usize)),
                        tile_size.x as usize,
                    ),
                    frame_size: tile_area as usize,
                });
            }
        }

        Self {
            tileset,
            tile_size: *tile_size,
            max_frame_count,
        }
    }
}

impl AnimationAsset for PxTilesetData {
    fn max_frame_count(&self) -> usize {
        self.max_frame_count
    }
}

impl PxTilesetData {
    pub fn tile_size(&self) -> UVec2 {
        self.tile_size
    }
}

/// A tileset for a tilemap. Create a [`Handle<PxTileset>`] with a [`PxAssets<PxTileset>`]
/// and an image file. The image file contains a column of tiles, ordered from bottom to top.
/// For animated tilesets, add additional frames to the right of tiles, marking the end
/// of an animation with a fully transparent tile or the end of the image.
/// See `assets/tileset/tileset.png` for an example.
pub type PxTileset = PxAsset<PxTilesetData>;

/// Creates a tilemap
#[derive(Bundle, Debug, Default)]
pub struct PxMapBundle<L: PxLayer> {
    /// A [`TilemapSize`] component
    pub size: TilemapSize,
    /// A [`TileStorage`] component
    pub storage: TileStorage,
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

/// Creates a tile
#[derive(Bundle, Debug, Default)]
pub struct PxTileBundle {
    /// A [`TileTexture`] component
    pub texture: TileTexture,
    /// A [`Visibility`] component
    pub visibility: Visibility,
}

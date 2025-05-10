use std::mem::replace;

use anyhow::{Error, Result};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    image::{CompressedImageFormats, ImageLoader, ImageLoaderSettings},
    render::{
        render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin},
        sync_component::SyncComponentPlugin,
        sync_world::RenderEntity,
        Extract, RenderApp,
    },
};
use serde::{Deserialize, Serialize};

use crate::{
    animation::{AnimatedAssetComponent, PxAnimation},
    image::PxImage,
    palette::asset_palette,
    position::{DefaultLayer, PxLayer, Spatial},
    prelude::*,
    sprite::PxSpriteAsset,
};

pub(crate) fn plug<L: PxLayer>(app: &mut App) {
    app.add_plugins((
        RenderAssetPlugin::<PxTileset>::default(),
        SyncComponentPlugin::<PxMap>::default(),
        SyncComponentPlugin::<PxTile>::default(),
    ))
    .init_asset::<PxTileset>()
    .init_asset_loader::<PxTilesetLoader>()
    .sub_app_mut(RenderApp)
    .add_systems(ExtractSchedule, (extract_maps::<L>, extract_tiles));
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

#[derive(Default)]
struct PxTilesetLoader;

impl AssetLoader for PxTilesetLoader {
    type Asset = PxTileset;
    type Settings = PxTilesetLoaderSettings;
    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &PxTilesetLoaderSettings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<PxTileset> {
        let image = ImageLoader::new(CompressedImageFormats::NONE)
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
                        .all(|&pixel| pixel == 0)
                {
                    tile.truncate(tile.len() - tile_area as usize);
                }

                let frame_count = tile.len() / tile_area as usize;
                if max_frame_count < frame_count {
                    max_frame_count = frame_count;
                }

                tileset.push(PxSpriteAsset {
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
#[derive(Asset, Clone, Reflect, Debug)]
pub struct PxTileset {
    pub(crate) tileset: Vec<PxSpriteAsset>,
    tile_size: UVec2,
    max_frame_count: usize,
}

impl RenderAsset for PxTileset {
    type SourceAsset = Self;
    type Param = ();

    fn prepare_asset(
        source_asset: Self,
        &mut (): &mut (),
    ) -> Result<Self, PrepareAssetError<Self>> {
        Ok(source_asset)
    }
}

impl PxTileset {
    /// The size of tiles in the tileset
    pub fn tile_size(&self) -> UVec2 {
        self.tile_size
    }
}

/// The tiles in a tilemap
#[derive(Clone, Default, Debug)]
pub struct PxTiles {
    tiles: Vec<Option<Entity>>,
    width: usize,
}

impl PxTiles {
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

impl<'a> Spatial for (&'a PxTiles, &'a PxTileset) {
    fn frame_size(&self) -> UVec2 {
        let (tiles, tileset) = self;
        tiles.size() * tileset.tile_size
    }
}

/// Creates a tilemap
#[derive(Component, Default, Clone, Debug)]
#[require(PxPosition, DefaultLayer, PxCanvas, Visibility)]
pub struct PxMap {
    /// The map's tiles
    pub tiles: PxTiles,
    /// The map's tileset
    pub tileset: Handle<PxTileset>,
}

impl AnimatedAssetComponent for PxMap {
    type Asset = PxTileset;

    fn handle(&self) -> &Handle<Self::Asset> {
        &self.tileset
    }

    fn max_frame_count(tileset: &PxTileset) -> usize {
        tileset.max_frame_count
    }
}

/// A tile. Must be added to tiles added to [`PxMap`].
#[derive(Component, Clone, Default, Debug)]
#[require(Visibility)]
pub struct PxTile {
    /// The index to the tile texture in the tileset
    pub texture: u32,
}

impl From<u32> for PxTile {
    fn from(value: u32) -> Self {
        Self { texture: value }
    }
}

pub(crate) type MapComponents<L> = (
    &'static PxMap,
    &'static PxPosition,
    &'static L,
    &'static PxCanvas,
    Option<&'static PxAnimation>,
    Option<&'static PxFilter>,
);

fn extract_maps<L: PxLayer>(
    maps: Extract<Query<(MapComponents<L>, &InheritedVisibility, RenderEntity)>>,
    render_entities: Extract<Query<RenderEntity>>,
    mut cmd: Commands,
) {
    for ((map, &position, layer, &canvas, animation, filter), visibility, id) in &maps {
        let mut entity = cmd.entity(id);

        if !visibility.get() {
            entity.remove::<L>();
            continue;
        }

        let mut map = map.clone();
        for opt_tile in &mut map.tiles.tiles {
            if let &mut Some(tile) = opt_tile {
                *opt_tile = render_entities.get(tile).ok();
            }
        }

        entity.insert((map, position, layer.clone(), canvas));

        if let Some(animation) = animation {
            entity.insert(*animation);
        } else {
            entity.remove::<PxAnimation>();
        }

        if let Some(filter) = filter {
            entity.insert(filter.clone());
        } else {
            entity.remove::<PxFilter>();
        }
    }
}

pub(crate) type TileComponents = (&'static PxTile, Option<&'static PxFilter>);

fn extract_tiles(
    tiles: Extract<Query<(TileComponents, &InheritedVisibility, RenderEntity)>>,
    mut cmd: Commands,
) {
    for ((tile, filter), visibility, entity) in &tiles {
        if !visibility.get() {
            // TODO This doesn't work
            continue;
        }

        let mut entity = cmd.entity(entity);
        entity.insert(tile.clone());

        if let Some(filter) = filter {
            entity.insert(filter.clone());
        } else {
            entity.remove::<PxFilter>();
        }
    }
}

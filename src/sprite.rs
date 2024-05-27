//! Sprites

use anyhow::{Error, Result};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    render::texture::{ImageLoader, ImageLoaderSettings},
    utils::BoxedFuture,
};
use serde::{Deserialize, Serialize};

use crate::{
    animation::{Animation, AnimationAsset},
    image::{PxImage, PxImageSliceMut},
    palette::asset_palette,
    pixel::Pixel,
    position::{PxLayer, Spatial},
    prelude::*,
};

pub(crate) fn sprite_plugin(app: &mut App) {
    app.init_asset::<PxSprite>()
        .init_asset_loader::<PxSpriteLoader>();
}

#[derive(Serialize, Deserialize)]
struct PxSpriteLoaderSettings {
    frame_count: usize,
    image_loader_settings: ImageLoaderSettings,
}

impl Default for PxSpriteLoaderSettings {
    fn default() -> Self {
        Self {
            frame_count: 1,
            image_loader_settings: default(),
        }
    }
}

struct PxSpriteLoader(ImageLoader);

impl FromWorld for PxSpriteLoader {
    fn from_world(world: &mut World) -> Self {
        Self(ImageLoader::from_world(world))
    }
}

impl AssetLoader for PxSpriteLoader {
    type Asset = PxSprite;
    type Settings = PxSpriteLoaderSettings;
    type Error = Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        settings: &'a PxSpriteLoaderSettings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<PxSprite>> {
        Box::pin(async move {
            let Self(image_loader) = self;
            let image = image_loader
                .load(reader, &settings.image_loader_settings, load_context)
                .await?;
            let palette = asset_palette().await;
            let data = PxImage::palette_indices(palette, &image)?;

            Ok(PxSprite {
                frame_size: data.area() / settings.frame_count,
                data,
            })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["px_sprite.png"]
    }
}

/// A sprite. Create a [`Handle<PxSprite>`] with a [`PxAssets<PxSprite>`] and an image.
/// If the sprite is animated, the frames should be laid out from bottom to top.
/// See `assets/sprite/runner.png` for an example of an animated sprite.
#[derive(Asset, Serialize, Deserialize, Reflect, Debug)]
pub struct PxSprite {
    // TODO Use 0 for transparency
    pub(crate) data: PxImage<Option<u8>>,
    pub(crate) frame_size: usize,
}

impl Animation for PxSprite {
    type Param = ();

    fn frame_count(&self) -> usize {
        self.data.area() / self.frame_size
    }

    fn draw(
        &self,
        _: (),
        image: &mut PxImageSliceMut<impl Pixel>,
        frame: impl Fn(UVec2) -> usize,
        filter: impl Fn(u8) -> u8,
    ) {
        let width = self.data.width();
        let image_width = image.image_width();
        image.for_each_mut(|slice_i, image_i, pixel| {
            if let Some(Some(value)) = self.data.get_pixel(IVec2::new(
                (slice_i % width) as i32,
                ((frame(UVec2::new(
                    (image_i % image_width) as u32,
                    (image_i / image_width) as u32,
                )) * self.frame_size
                    + slice_i)
                    / width) as i32,
            )) {
                pixel.set_value(filter(value));
            }
        });
    }
}

impl Spatial for PxSprite {
    fn frame_size(&self) -> UVec2 {
        UVec2::new(
            self.data.width() as u32,
            (self.frame_size / self.data.width()) as u32,
        )
    }
}

impl AnimationAsset for PxSprite {
    fn max_frame_count(&self) -> usize {
        self.frame_count()
    }
}

/// Spawns a sprite
#[derive(Bundle, Debug, Default)]
pub struct PxSpriteBundle<L: PxLayer> {
    /// A [`Handle<PxSprite>`] component
    pub sprite: Handle<PxSprite>,
    /// A [`PxPosition`] component
    pub position: PxPosition,
    /// A [`PxAnchor`] component
    pub anchor: PxAnchor,
    /// A layer component
    pub layer: L,
    /// A [`PxCanvas`] component
    pub canvas: PxCanvas,
    /// A [`Visibility`] component
    pub visibility: Visibility,
}

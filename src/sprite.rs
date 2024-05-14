//! Sprites

use crate::{
    animation::{Animation, AnimationAsset},
    asset::{PxAsset, PxAssetData},
    image::{PxImage, PxImageSliceMut},
    palette::Palette,
    pixel::Pixel,
    position::{PxLayer, Spatial},
    prelude::*,
};

/// Internal data for [`PxSprite`]
#[derive(Debug, Reflect)]
pub struct PxSpriteData {
    // TODO Use 0 for transparency
    pub(crate) data: PxImage<Option<u8>>,
    pub(crate) frame_size: usize,
}

impl PxAssetData for PxSpriteData {
    type Config = usize;

    fn new(palette: &Palette, image: &Image, frame_count: &Self::Config) -> Self {
        let data = PxImage::palette_indices(palette, image);

        Self {
            frame_size: data.area() / *frame_count,
            data,
        }
    }
}

impl Animation for PxSpriteData {
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

impl Spatial for PxSpriteData {
    fn frame_size(&self) -> UVec2 {
        UVec2::new(
            self.data.width() as u32,
            (self.frame_size / self.data.width()) as u32,
        )
    }
}

impl AnimationAsset for PxSpriteData {
    fn max_frame_count(&self) -> usize {
        self.frame_count()
    }
}

/// A sprite. Create a [`Handle<PxSprite>`] with a [`PxAssets<PxSprite>`] and an image.
/// If the sprite is animated, the frames should be laid out from bottom to top.
/// See `assets/sprite/runner.png` for an example of an animated sprite.
pub type PxSprite = PxAsset<PxSpriteData>;

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

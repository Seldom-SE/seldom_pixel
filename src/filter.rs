//! Filtering

use std::time::Duration;

use crate::{
    animation::{draw_animation, Animation, AnimationAsset},
    asset::{PxAsset, PxAssetData},
    image::{PxImage, PxImageSliceMut},
    palette::Palette,
    pixel::Pixel,
    position::PxLayer,
    prelude::*,
};

/// Internal data for a [`PxFilter`]
#[derive(Debug, Reflect)]
pub struct PxFilterData(pub(crate) PxImage<u8>);

impl PxAssetData for PxFilterData {
    const UUID: [u8; 16] = [
        52, 22, 15, 225, 67, 35, 43, 102, 203, 4, 164, 158, 160, 142, 197, 42,
    ];
    type Config = ();

    fn new(palette: &Palette, image: &Image, _: &Self::Config) -> Self {
        let indices = PxImage::palette_indices(palette, image);
        let mut filter = Vec::with_capacity(indices.area());
        let frame_size = palette.size;
        let frame_area = frame_size.x * frame_size.y;
        let filter_width = image.texture_descriptor.size.width;
        let frame_filter_width = filter_width / palette.size.x;

        let mut frame_visible = true;

        for i in 0..indices.area() {
            let frame_index = i as u32 / frame_area;
            let frame_pos = i as u32 % frame_area;

            if frame_pos == 0 {
                if !frame_visible {
                    for _ in 0..frame_area {
                        filter.pop();
                    }
                    break;
                }

                frame_visible = false;
            }

            filter.push(
                if let Some(index) = indices.pixel(
                    (UVec2::new(
                        frame_index % frame_filter_width,
                        frame_index / frame_filter_width,
                    ) * frame_size
                        + UVec2::new(frame_pos % frame_size.x, frame_pos / frame_size.x))
                    .as_ivec2(),
                ) {
                    frame_visible = true;
                    index
                } else {
                    0
                },
            );
        }

        Self(PxImage::new(filter, frame_area as usize))
    }
}

impl Animation for PxFilterData {
    type Param = ();

    fn frame_count(&self) -> usize {
        let Self(filter) = self;
        filter.area() / filter.width()
    }

    fn draw(
        &self,
        _: Self::Param,
        image: &mut PxImageSliceMut<impl Pixel>,
        frame: impl Fn(UVec2) -> usize,
        _: impl Fn(u8) -> u8,
    ) {
        let Self(filter) = self;
        let width = image.width();
        image.for_each_mut(|index, _, pixel| {
            if let Some(pixel) = pixel.get_value_mut() {
                let index = index as u32;
                *pixel = filter.pixel(IVec2::new(
                    *pixel as i32,
                    frame(UVec2::new(index % width, index / width)) as i32,
                ));
            }
        })
    }
}

impl AnimationAsset for PxFilterData {
    fn max_frame_count(&self) -> usize {
        self.frame_count()
    }
}

impl PxFilterData {
    pub(crate) fn as_fn(&self) -> impl '_ + Fn(u8) -> u8 {
        let Self(filter) = self;
        |pixel| filter.pixel(IVec2::new(pixel as i32, 0))
    }
}

/// Maps colors of an image to different colors. Filter a single sprite, text, or tilemap
/// by adding a [`Handle<PxFilter>`] to it, or filter entire layers
/// by spawning a [`PxFilterBundle`]. Create a [`Handle<PxFilter>`] with a [`PxAssets<PxFilter>`]
/// and an image file. The image should have pixels in the same positions as the palette.
/// The position of each pixel describes the mapping of colors. The image must only contain colors
/// that are also in the palette. For animated filters, arrange a number of filters
/// from the bottom-left corner, moving rightwards, wrapping upwards when it gets to the edge
/// of the image. For examples, see the `assets/` directory in this repository. `fade_to_black.png`
/// is an animated filter.
pub type PxFilter = PxAsset<PxFilterData>;

/// Determines which layers a filter appies to
#[derive(Component)]
pub enum PxFilterLayers<L: PxLayer> {
    /// Filter applies to a single layer
    Single {
        /// Layer the filter appies to
        layer: L,
        /// If `true`, the filter will apply only to the entities on that layer,
        /// before it is rendered. If `false`, it will apply to the entire image when the layer
        /// is rendered, including the background color.
        clip: bool,
    },
    /// Filter applies to a set list of layers
    Many(Vec<L>),
    /// Filter applies to layers selected by the given function
    Select(Box<dyn Fn(&L) -> bool + Send + Sync>),
}

impl<L: PxLayer> Default for PxFilterLayers<L> {
    fn default() -> Self {
        Self::single_clip(default())
    }
}

impl<L: PxLayer, T: 'static + Fn(&L) -> bool + Send + Sync> From<T> for PxFilterLayers<L> {
    fn from(t: T) -> Self {
        Self::Select(Box::new(t))
    }
}

impl<L: PxLayer> PxFilterLayers<L> {
    /// Creates a [`PxFilterLayers::Single`] with the given layer, with clip enabled
    pub fn single_clip(layer: L) -> Self {
        Self::Single { layer, clip: true }
    }

    /// Creates a [`PxFilterLayers::Single`] with the given layer, with clip disabled
    pub fn single_over(layer: L) -> Self {
        Self::Single { layer, clip: false }
    }
}

/// Makes a filter that applies to entire layers
#[derive(Bundle, Default)]
pub struct PxFilterBundle<L: PxLayer> {
    /// A [`Handle<PxFilter>`] component
    pub filter: Handle<PxFilter>,
    /// A [`PxFilterLayers`] component
    pub layers: PxFilterLayers<L>,
    /// A [`Visibility`] component
    pub visibility: Visibility,
}

pub(crate) fn draw_filter(
    filter: &PxFilterData,
    animation: Option<(
        PxAnimationDirection,
        PxAnimationDuration,
        PxAnimationFinishBehavior,
        PxAnimationFrameTransition,
        Duration,
    )>,
    image: &mut PxImageSliceMut<impl Pixel>,
) {
    draw_animation(filter, (), image, animation, []);
}

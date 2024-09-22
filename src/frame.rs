//! Frames

use anyhow::{Error, Result};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    render::{
        render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin},
        texture::{ImageLoader, ImageLoaderSettings},
        Extract, RenderApp,
    },
    tasks::{ComputeTaskPool, ParallelSliceMut},
};
use kiddo::{ImmutableKdTree, SquaredEuclidean};
use serde::{Deserialize, Serialize};

use crate::{
    animation::{AnimationComponents, Drawable},
    image::{PxImage, PxImageSliceMut},
    palette::{asset_palette, PaletteParam},
    pixel::Pixel,
    position::{PxLayer, PxOffset, PxSize, Spatial},
    prelude::*,
};

pub(crate) fn plug<L: PxLayer>(app: &mut App) {
    app.add_plugins(RenderAssetPlugin::<PxFrame>::default())
        .init_asset::<PxFrame>()
        .init_asset_loader::<PxFrameLoader>()
        .add_systems(PostUpdate, image_to_frame)
        .sub_app_mut(RenderApp)
        .add_systems(ExtractSchedule, extract_frames::<L>);
}

#[derive(Serialize, Deserialize)]
struct PxFrameLoaderSettings {
    image_loader_settings: ImageLoaderSettings,
}

impl Default for PxFrameLoaderSettings {
    fn default() -> Self {
        Self {
            image_loader_settings: default(),
        }
    }
}

struct PxFrameLoader(ImageLoader);

impl FromWorld for PxFrameLoader {
    fn from_world(world: &mut World) -> Self {
        Self(ImageLoader::from_world(world))
    }
}

impl AssetLoader for PxFrameLoader {
    type Asset = PxFrame;
    type Settings = PxFrameLoaderSettings;
    type Error = Error;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        settings: &'a PxFrameLoaderSettings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<PxFrame> {
        let Self(image_loader) = self;
        let image = image_loader
            .load(reader, &settings.image_loader_settings, load_context)
            .await?;
        let palette = asset_palette().await;
        let data = PxImage::palette_indices(palette, &image)?;

        Ok(PxFrame { data })
    }

    fn extensions(&self) -> &[&str] {
        &["px_frame.png"]
    }
}

/// A frame. Create a [`Handle<PxFrame>`] with a [`PxAssets<PxFrame>`] and an image.
/// If the frame is animated, the frames should be laid out from bottom to top.
/// See `assets/frame/runner.png` for an example of an animated frame.
#[derive(Asset, Serialize, Deserialize, Clone, Reflect, Debug)]
pub struct PxFrame {
    // TODO Use 0 for transparency
    pub(crate) data: PxImage<Option<u8>>,
}

impl RenderAsset for PxFrame {
    type SourceAsset = Self;
    type Param = ();

    fn prepare_asset(
        source_asset: Self,
        &mut (): &mut (),
    ) -> Result<Self, PrepareAssetError<Self>> {
        Ok(source_asset)
    }
}

impl Drawable for PxFrame {
    type Param = ();

    fn draw(
        &self,
        _: (),
        canvas_slice: &mut PxImageSliceMut<impl Pixel>,
        offset: UVec2,
        size: UVec2,
        filter: impl Fn(u8) -> u8,
    ) {
        let image_width = self.data.width();
        let image_height = self.data.height();

        let offset_x = offset.x;
        let offset_y = offset.y;
        let size_x = size.x as usize;
        let size_y = size.y as usize;

        if offset_x as usize + size_x > image_width as usize {
            eprintln!(
                "Error: Requested offset + size on X axis ({} + {}) exceeds the image width ({})",
                offset_x, size_x, image_width
            );
            return;
        }

        if offset_y as usize + size_y > image_height as usize {
            eprintln!(
                "Error: Requested offset + size on Y axis ({} + {}) exceeds the image height ({})",
                offset_y, size_y, image_height
            );
            return;
        }

        canvas_slice.for_each_mut(|slice_i, _, pixel| {
            let slice_x = (slice_i % size_x) as u32;
            let slice_y = (slice_i / size_x) as u32;

            let pixel_pos = IVec2::new(slice_x + offset_x, slice_y + offset_y);

            if let Some(Some(value)) = self.data.get_pixel(pixel_pos) {
                pixel.set_value(filter(value));
            }
        });
    }
}

/// Spawns a sprite
#[derive(Bundle, Debug, Default)]
pub struct PxFrameBundle<L: PxLayer> {
    /// A [`Handle<PxFrame>`] component
    pub frame: Handle<PxFrame>,
    /// A [`PxOffset`] component
    pub offset: PxOffset,
    /// A [`PxSize`] component
    pub size: PxSize,
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
    /// An [`InheritedVisibility`] component
    pub inherited_visibility: InheritedVisibility,
}

fn srgb_to_linear(c: f32) -> f32 {
    if c >= 0.04045 {
        ((c + 0.055) / (1. + 0.055)).powf(2.4)
    } else {
        c / 12.92
    }
}

#[allow(clippy::excessive_precision)]
fn srgb_to_oklab(rd: f32, gn: f32, bu: f32) -> (f32, f32, f32) {
    let rd = srgb_to_linear(rd);
    let gn = srgb_to_linear(gn);
    let bu = srgb_to_linear(bu);

    let l = 0.4122214708 * rd + 0.5363325363 * gn + 0.0514459929 * bu;
    let m = 0.2119034982 * rd + 0.6806995451 * gn + 0.1073969566 * bu;
    let s = 0.0883024619 * rd + 0.2817188376 * gn + 0.6299787005 * bu;

    let lp = l.cbrt();
    let mp = m.cbrt();
    let sp = s.cbrt();

    (
        0.2104542553 * lp + 0.7936177850 * mp - 0.0040720468 * sp,
        1.9779984951 * lp - 2.4285922050 * mp + 0.4505937099 * sp,
        0.0259040371 * lp + 0.7827717662 * mp - 0.8086757660 * sp,
    )
}

/// Size of threshold map to use for dithering. The image is tiled with dithering according to this
/// map, so smaller sizes will have more visible repetition and worse color approximation, but
/// larger sizes are much, much slower with pattern dithering.
#[derive(Clone, Copy)]
pub enum ThresholdMap {
    /// 2x2
    X2_2,
    /// 4x4
    X4_4,
    /// 8x8
    X8_8,
}

/// Dithering algorithm. Perf measurements are for 10,000 pixels with a 4x4 threshold map on a
/// pretty old machine.
#[derive(Clone, Copy)]
pub enum DitherAlgorithm {
    /// Almost as fast as undithered. 16.0 ms in debug mode and 1.23 ms in release mode. Doesn't
    /// make very good use of the color palette.
    Ordered,
    /// Slow, but mixes colors very well. 219 ms in debug mode and 6.81 ms in release mode. Consider
    /// only using this algorithm with some optimizations enabled.
    Pattern,
}

/// Info needed to dither an image
pub struct Dither {
    /// Dithering algorithm
    pub algorithm: DitherAlgorithm,
    /// How much to dither. Lower values leave solid color areas. Should range from 0 to 1.
    pub threshold: f32,
    /// Threshold map size
    pub threshold_map: ThresholdMap,
}

/// Renders the contents of an image to a sprite every tick. The image is interpreted as
/// `Rgba8UnormSrgb`.
#[derive(Component)]
pub struct ImageToFrame {
    /// Image to render
    pub image: Handle<Image>,
    /// Dithering
    pub dither: Option<Dither>,
}

trait MapSize<const SIZE: usize> {
    const WIDTH: usize;
    const MAP: [usize; SIZE];
}

impl MapSize<1> for () {
    const WIDTH: usize = 1;
    const MAP: [usize; 1] = [0];
}

impl MapSize<4> for () {
    const WIDTH: usize = 2;
    #[rustfmt::skip]
    const MAP: [usize; 4] = [
        0, 2,
        3, 1,
    ];
}

impl MapSize<16> for () {
    const WIDTH: usize = 4;
    #[rustfmt::skip]
    const MAP: [usize; 16] = [
        0, 8, 2, 10,
        12, 4, 14, 6,
        3, 11, 1, 9,
        15, 7, 13, 5,
    ];
}

impl MapSize<64> for () {
    const WIDTH: usize = 8;
    #[rustfmt::skip]
    const MAP: [usize; 64] = [
        0, 48, 12, 60, 3, 51, 15, 63,
        32, 16, 44, 28, 35, 19, 47, 31,
        8, 56, 4, 52, 11, 59, 7, 55,
        40, 24, 36, 20, 43, 27, 39, 23,
        2, 50, 14, 62, 1, 49, 13, 61,
        34, 18, 46, 30, 33, 17, 45, 29,
        10, 58, 6, 54, 9, 57, 5, 53,
        42, 26, 38, 22, 41, 25, 37, 21,
    ];
}

trait Algorithm<const MAP_SIZE: usize> {
    fn compute(
        color: Vec3,
        threshold: Vec3,
        threshold_index: usize,
        candidates: &mut [usize; MAP_SIZE],
        palette_tree: &ImmutableKdTree<f32, 3>,
        palette: &[Vec3],
    ) -> u8;
}

enum ClosestAlg {}

impl<const MAP_SIZE: usize> Algorithm<MAP_SIZE> for ClosestAlg {
    fn compute(
        color: Vec3,
        _: Vec3,
        _: usize,
        _: &mut [usize; MAP_SIZE],
        palette_tree: &ImmutableKdTree<f32, 3>,
        _: &[Vec3],
    ) -> u8 {
        palette_tree
            .approx_nearest_one::<SquaredEuclidean>(&color.into())
            .item as usize as u8
    }
}

enum OrderedAlg {}

impl<const MAP_SIZE: usize> Algorithm<MAP_SIZE> for OrderedAlg {
    fn compute(
        color: Vec3,
        threshold: Vec3,
        threshold_index: usize,
        _: &mut [usize; MAP_SIZE],
        palette_tree: &ImmutableKdTree<f32, 3>,
        _: &[Vec3],
    ) -> u8 {
        palette_tree
            .approx_nearest_one::<SquaredEuclidean>(
                &(color + threshold * (threshold_index as f32 / MAP_SIZE as f32 - 0.5)).into(),
            )
            .item as u8
    }
}

enum PatternAlg {}

impl<const MAP_SIZE: usize> Algorithm<MAP_SIZE> for PatternAlg {
    fn compute(
        color: Vec3,
        threshold: Vec3,
        threshold_index: usize,
        candidates: &mut [usize; MAP_SIZE],
        palette_tree: &ImmutableKdTree<f32, 3>,
        palette: &[Vec3],
    ) -> u8 {
        let mut error = Vec3::ZERO;
        for candidate_ref in &mut *candidates {
            let sample = color + error * threshold;
            let candidate = palette_tree
                .approx_nearest_one::<SquaredEuclidean>(&sample.into())
                .item as usize;

            *candidate_ref = candidate;
            error += color - palette[candidate];
        }

        candidates.sort_unstable_by(|&candidate_1, &candidate_2| {
            palette[candidate_1][0].total_cmp(&palette[candidate_2][0])
        });

        candidates[threshold_index] as u8
    }
}

fn dither_slice<A: Algorithm<MAP_SIZE>, const MAP_SIZE: usize>(
    pixels: &mut [(usize, (&[u8], &mut Option<u8>))],
    threshold: f32,
    size: UVec2,
    palette_tree: &ImmutableKdTree<f32, 3>,
    palette: &[Vec3],
) where
    (): MapSize<MAP_SIZE>,
{
    let mut candidates = [0; MAP_SIZE];

    for &mut (i, (color, ref mut pixel)) in pixels {
        let i = i as u32;
        let pos = UVec2::new(i % size.x, i / size.x);

        if color[3] == 0 {
            **pixel = None;
            continue;
        }

        **pixel = Some(A::compute(
            Vec3::from(srgb_to_oklab(
                color[0] as f32 / 255.,
                color[1] as f32 / 255.,
                color[2] as f32 / 255.,
            )),
            Vec3::splat(threshold),
            <() as MapSize<MAP_SIZE>>::MAP[pos.x as usize % <() as MapSize<MAP_SIZE>>::WIDTH
                * <() as MapSize<MAP_SIZE>>::WIDTH
                + pos.y as usize % <() as MapSize<MAP_SIZE>>::WIDTH],
            &mut candidates,
            palette_tree,
            palette,
        ));
    }
}

// TODO Use more helpers
// TODO Feature gate
// TODO Immediate function version
fn image_to_frame(
    mut to_frames: Query<(&ImageToFrame, &mut Handle<PxFrame>)>,
    images: Res<Assets<Image>>,
    palette: PaletteParam,
    mut sprites: ResMut<Assets<PxFrame>>,
) {
    if to_frames.iter().next().is_none() {
        return;
    }

    let Some(palette) = palette.get() else {
        return;
    };

    let palette = palette
        .colors
        .iter()
        .map(|&[r, g, b]| srgb_to_oklab(r as f32 / 255., g as f32 / 255., b as f32 / 255.).into())
        .collect::<Vec<Vec3>>();

    let palette_tree = ImmutableKdTree::from(
        &palette
            .iter()
            .map(|&color| color.into())
            .collect::<Vec<[f32; 3]>>()[..],
    );

    to_frames.iter_mut().for_each(|(image, mut sprite)| {
        let dither = &image.dither;
        let image = images.get(&image.image).unwrap();

        if *sprite == Handle::default() {
            let data = PxImage::empty_from_image(image);

            *sprite = sprites.add(PxFrame { data });
        }

        let sprite = sprites.get_mut(&*sprite).unwrap();

        let size = image.texture_descriptor.size;
        let size = UVec2::new(size.width, size.height);
        if sprite.data.size() != size {
            let data = PxImage::empty_from_image(image);

            // sprite.frame_size = data.area();
            sprite.data = data;
        }

        let mut pixels = image
            .data
            .chunks_exact(4)
            .zip(sprite.data.iter_mut())
            .enumerate()
            .collect::<Vec<_>>();

        pixels.par_chunk_map_mut(ComputeTaskPool::get(), 20, |_, pixels| {
            use DitherAlgorithm::*;
            use ThresholdMap::*;

            match *dither {
                None => dither_slice::<ClosestAlg, 1>(pixels, 0., size, &palette_tree, &palette),
                Some(Dither {
                    algorithm: Ordered,
                    threshold,
                    threshold_map: X2_2,
                }) => {
                    dither_slice::<OrderedAlg, 4>(pixels, threshold, size, &palette_tree, &palette)
                }
                Some(Dither {
                    algorithm: Ordered,
                    threshold,
                    threshold_map: X4_4,
                }) => {
                    dither_slice::<OrderedAlg, 16>(pixels, threshold, size, &palette_tree, &palette)
                }
                Some(Dither {
                    algorithm: Ordered,
                    threshold,
                    threshold_map: X8_8,
                }) => {
                    dither_slice::<OrderedAlg, 64>(pixels, threshold, size, &palette_tree, &palette)
                }
                Some(Dither {
                    algorithm: Pattern,
                    threshold,
                    threshold_map: X2_2,
                }) => {
                    dither_slice::<PatternAlg, 4>(pixels, threshold, size, &palette_tree, &palette)
                }
                Some(Dither {
                    algorithm: Pattern,
                    threshold,
                    threshold_map: X4_4,
                }) => {
                    dither_slice::<PatternAlg, 16>(pixels, threshold, size, &palette_tree, &palette)
                }
                Some(Dither {
                    algorithm: Pattern,
                    threshold,
                    threshold_map: X8_8,
                }) => {
                    dither_slice::<PatternAlg, 64>(pixels, threshold, size, &palette_tree, &palette)
                }
            }
        });
    });
}

pub(crate) type FrameComponents<L> = (
    &'static Handle<PxFrame>,
    &'static PxPosition,
    &'static PxOffset,
    &'static PxSize,
    &'static PxAnchor,
    &'static L,
    &'static PxCanvas,
    Option<&'static Handle<PxFilter>>,
);

fn extract_frames<L: PxLayer>(
    frames: Extract<Query<(FrameComponents<L>, &InheritedVisibility)>>,
    mut cmd: Commands,
) {
    for ((frame, &position, &offset, &size, &anchor, layer, &canvas, filter), visibility) in &frames
    {
        if !visibility.get() {
            continue;
        }

        let mut frame = cmd.spawn((
            frame.clone(),
            position,
            offset,
            size,
            anchor,
            layer.clone(),
            canvas,
        ));

        // if let Some((&direction, &duration, &on_finish, &frame_transition, &start)) = animation {
        //     frame.insert((direction, duration, on_finish, frame_transition, start));
        // }

        if let Some(filter) = filter {
            frame.insert(filter.clone());
        }
    }
}

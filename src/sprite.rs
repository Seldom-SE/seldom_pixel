//! Sprites

use anyhow::{Error, Result};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    render::texture::{ImageLoader, ImageLoaderSettings},
    tasks::{ComputeTaskPool, ParallelSliceMut},
    utils::BoxedFuture,
};
use kiddo::{ImmutableKdTree, SquaredEuclidean};
use serde::{Deserialize, Serialize};

use crate::{
    animation::{Animation, AnimationAsset},
    image::{PxImage, PxImageSliceMut},
    palette::{asset_palette, PaletteParam},
    pixel::Pixel,
    position::{PxLayer, Spatial},
    prelude::*,
    set::PxSet,
};

pub(crate) fn sprite_plugin(app: &mut App) {
    app.init_asset::<PxSprite>()
        .init_asset_loader::<PxSpriteLoader>()
        .add_systems(PostUpdate, image_to_sprite.before(PxSet::Draw));
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
pub struct ImageToSprite {
    /// Image to render
    pub image: Handle<Image>,
    /// Dithering
    pub dither: Option<Dither>,
}

// const MAP_SIZE: usize = THRESHOLD_MAP.len();

trait MapSize<const MAP_SIZE: usize> {
    fn width() -> usize;
    fn map() -> [usize; MAP_SIZE];
}

impl MapSize<4> for () {
    fn width() -> usize {
        2
    }

    #[rustfmt::skip]
    fn map() -> [usize; 4] {
        [
            0, 2,
            3, 1,
        ]
    }
}

impl MapSize<16> for () {
    fn width() -> usize {
        4
    }

    #[rustfmt::skip]
    fn map() -> [usize; 16] {
        [
            0, 8, 2, 10,
            12, 4, 14, 6,
            3, 11, 1, 9,
            15, 7, 13, 5,
        ]
    }
}

impl MapSize<64> for () {
    fn width() -> usize {
        8
    }

    #[rustfmt::skip]
    fn map() -> [usize; 64] {
        [
            0, 48, 12, 60, 3, 51, 15, 63,
            32, 16, 44, 28, 35, 19, 47, 31,
            8, 56, 4, 52, 11, 59, 7, 55,
            40, 24, 36, 20, 43, 27, 39, 23,
            2, 50, 14, 62, 1, 49, 13, 61,
            34, 18, 46, 30, 33, 17, 45, 29,
            10, 58, 6, 54, 9, 57, 5, 53,
            42, 26, 38, 22, 41, 25, 37, 21,
        ]
    }
}

fn dither<const MAP_SIZE: usize, const ALGORITHM: usize>(pixels: &mut [(usize, (&[u8], &mut Option<u8>))]) where (): MapSize<MAP_SIZE> {
    let threshold_map = <() as MapSize<MAP_SIZE>>::map();

    let mut candidates = [0; MAP_SIZE];

    let threshold_map = match *dither {
        None => Vec::new(),
        Some(Dither {  })
    }

    for &mut (i, (color, ref mut pixel)) in pixels {
        let i = i as u32;
        let pos = UVec2::new(i % size.x, i / size.x);

        if color[3] == 0 {
            **pixel = None;
            continue;
        }

        let color = Vec3::from(srgb_to_oklab(
            color[0] as f32 / 255.,
            color[1] as f32 / 255.,
            color[2] as f32 / 255.,
        ));

        **pixel = Some(match *dither {
            None => {
                palette_tree
                    .approx_nearest_one::<SquaredEuclidean>(&color.into())
                    .item as usize as u8
            }
            Some(Dither {
                algorithm: DitherAlgorithm::Ordered,
                threshold,
                threshold_map,
            }) => {
                palette_tree
                    .approx_nearest_one::<SquaredEuclidean>(
                        &(color
                            + Vec3::splat(0.04)
                                * (THRESHOLD_MAP[pos.x as usize % MAP_SIZE]
                                    [pos.y as usize % MAP_SIZE]
                                    as f32
                                    / (MAP_SIZE * MAP_SIZE) as f32
                                    - 0.5))
                            .into(),
                    )
                    .item as u8
            }
            Some(Dither {
                algorithm: DitherAlgorithm::Pattern,
                threshold,
                threshold_map,
            }) => {
                let mut error = Vec3::ZERO;
                for candidate_ref in &mut candidates {
                    let sample = color + error * Vec3::splat(0.1);
                    let candidate = palette_tree
                        .approx_nearest_one::<SquaredEuclidean>(&sample.into())
                        .item as usize;

                    *candidate_ref = candidate;
                    error += color - palette[candidate];
                }

                candidates.sort_unstable_by(|&candidate_1, &candidate_2| {
                    palette[candidate_1][0].total_cmp(&palette[candidate_2][0])
                });

                let index =
                    THRESHOLD_MAP[pos.x as usize % MAP_SIZE][pos.y as usize % MAP_SIZE];
                candidates[index] as u8
            }
        });
    }
}

// TODO Use more helpers
fn image_to_sprite(
    mut to_sprites: Query<(&ImageToSprite, &mut Handle<PxSprite>)>,
    images: Res<Assets<Image>>,
    palette: PaletteParam,
    mut sprites: ResMut<Assets<PxSprite>>,
) {
    let span = info_span!("init", name = "init").entered();
    if to_sprites.iter().next().is_none() {
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
    drop(span);

    to_sprites.iter_mut().for_each(|(image, mut sprite)| {
        let span = info_span!("making_images", name = "making_images").entered();
        let dither = &image.dither;
        let image = images.get(&image.image).unwrap();

        if *sprite == Handle::default() {
            let data = PxImage::empty_from_image(image);

            *sprite = sprites.add(PxSprite {
                frame_size: data.area(),
                data,
            });
        }

        let sprite = sprites.get_mut(&*sprite).unwrap();

        let size = image.texture_descriptor.size;
        let size = UVec2::new(size.width, size.height);
        if sprite.data.size() != size {
            let data = PxImage::empty_from_image(image);

            sprite.frame_size = data.area();
            sprite.data = data;
        }

        let mut pixels = image
            .data
            .chunks_exact(4)
            .zip(sprite.data.iter_mut())
            .enumerate()
            .collect::<Vec<_>>();
        drop(span);

        pixels.par_chunk_map_mut(ComputeTaskPool::get(), 20, |pixels| {
        });
    });
}

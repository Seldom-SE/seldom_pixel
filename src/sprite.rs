//! Sprites

use std::error::Error;

use bevy_asset::{AssetLoader, LoadContext, io::Reader};
use bevy_derive::{Deref, DerefMut};
use bevy_image::{CompressedImageFormats, ImageLoader, ImageLoaderSettings};
use bevy_math::{ivec2, uvec2};
use bevy_render::{
    Extract, RenderApp,
    render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin},
    sync_component::SyncComponentPlugin,
    sync_world::RenderEntity,
};
use serde::{Deserialize, Serialize};

use crate::{
    animation::{AnimatedAssetComponent, Frames},
    image::{PxImage, PxImageSliceMut},
    palette::asset_palette,
    position::{DefaultLayer, PxLayer, Spatial},
    prelude::*,
};

pub(crate) fn plug<L: PxLayer>(app: &mut App) {
    app.add_plugins((
        RenderAssetPlugin::<PxSpriteAsset>::default(),
        SyncComponentPlugin::<PxSprite>::default(),
    ))
    .init_asset::<PxSpriteAsset>()
    .init_asset_loader::<PxSpriteLoader>()
    .sub_app_mut(RenderApp)
    .add_systems(
        ExtractSchedule,
        (
            extract_sprites::<L>,
            // extract_image_to_sprites::<L>
        ),
    );
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

#[derive(Default)]
struct PxSpriteLoader;

impl AssetLoader for PxSpriteLoader {
    type Asset = PxSpriteAsset;
    type Settings = PxSpriteLoaderSettings;
    type Error = Box<dyn Error + Send + Sync>;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &PxSpriteLoaderSettings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<PxSpriteAsset, Self::Error> {
        let image = ImageLoader::new(CompressedImageFormats::NONE)
            .load(reader, &settings.image_loader_settings, load_context)
            .await?;
        let palette = asset_palette().await;
        let data = PxImage::palette_indices(palette, &image).map_err(|err| err.to_string())?;

        Ok(PxSpriteAsset {
            frame_size: data.area() / settings.frame_count,
            data,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["px_sprite.png"]
    }
}

/// A sprite. Create a [`Handle<PxSpriteAsset>`] with a [`PxAssets<PxSprite>`] and an image.
/// If the sprite is animated, the frames should be laid out from top to bottom.
/// See `assets/sprite/runner.png` for an example of an animated sprite.
#[derive(Asset, Serialize, Deserialize, Clone, Reflect, Debug)]
pub struct PxSpriteAsset {
    pub(crate) data: PxImage,
    pub(crate) frame_size: usize,
}

impl RenderAsset for PxSpriteAsset {
    type SourceAsset = Self;
    type Param = ();

    fn prepare_asset(
        source_asset: Self,
        _: AssetId<Self>,
        &mut (): &mut (),
        _: Option<&Self>,
    ) -> Result<Self, PrepareAssetError<Self>> {
        Ok(source_asset)
    }
}

impl Frames for PxSpriteAsset {
    type Param = ();

    fn frame_count(&self) -> usize {
        self.data.area() / self.frame_size
    }

    fn draw(
        &self,
        (): (),
        image: &mut PxImageSliceMut,
        frame: impl Fn(UVec2) -> usize,
        filter: impl Fn(u8) -> u8,
    ) {
        let width = self.data.width();
        let image_width = image.image_width();
        image.for_each_mut(|slice_i, image_i, pixel| {
            if let Some(value) = self.data.get_pixel(ivec2(
                (slice_i % width) as i32,
                ((frame(uvec2(
                    (image_i % image_width) as u32,
                    (image_i / image_width) as u32,
                )) * self.frame_size
                    + slice_i)
                    / width) as i32,
            )) && value != 0
            {
                *pixel = filter(value);
            }
        });
    }
}

impl Spatial for PxSpriteAsset {
    fn frame_size(&self) -> UVec2 {
        UVec2::new(
            self.data.width() as u32,
            (self.frame_size / self.data.width()) as u32,
        )
    }
}

/// A sprite
#[derive(Component, Deref, DerefMut, Default, Clone, Debug)]
#[require(PxPosition, PxAnchor, DefaultLayer, PxCanvas, Visibility)]
pub struct PxSprite(pub Handle<PxSpriteAsset>);

impl From<Handle<PxSpriteAsset>> for PxSprite {
    fn from(value: Handle<PxSpriteAsset>) -> Self {
        Self(value)
    }
}

impl AnimatedAssetComponent for PxSprite {
    type Asset = PxSpriteAsset;

    fn handle(&self) -> &Handle<Self::Asset> {
        self
    }

    fn max_frame_count(sprite: &PxSpriteAsset) -> usize {
        sprite.frame_count()
    }
}

// /// Size of threshold map to use for dithering. The image is tiled with dithering according to this
// /// map, so smaller sizes will have more visible repetition and worse color approximation, but
// /// larger sizes are much, much slower with pattern dithering.
// #[derive(Clone, Copy, Debug)]
// pub enum ThresholdMap {
//     /// 2x2
//     X2_2,
//     /// 4x4
//     X4_4,
//     /// 8x8
//     X8_8,
// }
//
// /// Dithering algorithm. Perf measurements are for 10,000 pixels with a 4x4 threshold map on a
// /// pretty old machine.
// #[derive(Clone, Copy, Debug)]
// pub enum DitherAlgorithm {
//     /// Almost as fast as undithered. 16.0 ms in debug mode and 1.23 ms in release mode. Doesn't
//     /// make very good use of the color palette.
//     Ordered,
//     /// Slow, but mixes colors very well. 219 ms in debug mode and 6.81 ms in release mode. Consider
//     /// only using this algorithm with some optimizations enabled.
//     Pattern,
// }
//
// /// Info needed to dither an image
// #[derive(Clone, Debug)]
// pub struct Dither {
//     /// Dithering algorithm
//     pub algorithm: DitherAlgorithm,
//     /// How much to dither. Lower values leave solid color areas. Should range from 0 to 1.
//     pub threshold: f32,
//     /// Threshold map size
//     pub threshold_map: ThresholdMap,
// }

// // TODO Example
// /// Renders the contents of an image to a sprite every tick. The image is interpreted as
// /// `Rgba8UnormSrgb`.
// #[derive(Component, Clone, Default, Debug)]
// pub struct ImageToSprite {
//     /// Image to render
//     pub image: Handle<Image>,
//     /// Dithering
//     pub dither: Option<Dither>,
// }

// /// Spawns a sprite generated from an [`Image`]
// #[derive(Bundle, Debug, Default)]
// pub struct ImageToSpriteBundle<L: PxLayer> {
//     /// A [`Handle<PxSprite>`] component
//     pub image: ImageToSprite,
//     /// A [`PxPosition`] component
//     pub position: PxPosition,
//     /// A [`PxAnchor`] component
//     pub anchor: PxAnchor,
//     /// A layer component
//     pub layer: L,
//     /// A [`PxCanvas`] component
//     pub canvas: PxCanvas,
//     /// A [`Visibility`] component
//     pub visibility: Visibility,
//     /// An [`InheritedVisibility`] component
//     pub inherited_visibility: InheritedVisibility,
// }

// pub(crate) trait MapSize<const SIZE: usize> {
//     const WIDTH: usize;
//     const MAP: [usize; SIZE];
// }
//
// impl MapSize<1> for () {
//     const WIDTH: usize = 1;
//     const MAP: [usize; 1] = [0];
// }
//
// impl MapSize<4> for () {
//     const WIDTH: usize = 2;
//     #[rustfmt::skip]
//     const MAP: [usize; 4] = [
//         0, 2,
//         3, 1,
//     ];
// }
//
// impl MapSize<16> for () {
//     const WIDTH: usize = 4;
//     #[rustfmt::skip]
//     const MAP: [usize; 16] = [
//         0, 8, 2, 10,
//         12, 4, 14, 6,
//         3, 11, 1, 9,
//         15, 7, 13, 5,
//     ];
// }
//
// impl MapSize<64> for () {
//     const WIDTH: usize = 8;
//     #[rustfmt::skip]
//     const MAP: [usize; 64] = [
//         0, 48, 12, 60, 3, 51, 15, 63,
//         32, 16, 44, 28, 35, 19, 47, 31,
//         8, 56, 4, 52, 11, 59, 7, 55,
//         40, 24, 36, 20, 43, 27, 39, 23,
//         2, 50, 14, 62, 1, 49, 13, 61,
//         34, 18, 46, 30, 33, 17, 45, 29,
//         10, 58, 6, 54, 9, 57, 5, 53,
//         42, 26, 38, 22, 41, 25, 37, 21,
//     ];
// }
//
// pub(crate) trait Algorithm<const MAP_SIZE: usize> {
//     fn compute(
//         color: Vec3,
//         threshold: Vec3,
//         threshold_index: usize,
//         candidates: &mut [usize; MAP_SIZE],
//         palette_tree: &ImmutableKdTree<f32, 3>,
//         palette: &[Vec3],
//     ) -> u8;
// }
//
// pub(crate) enum ClosestAlg {}
//
// impl<const MAP_SIZE: usize> Algorithm<MAP_SIZE> for ClosestAlg {
//     fn compute(
//         color: Vec3,
//         _: Vec3,
//         _: usize,
//         _: &mut [usize; MAP_SIZE],
//         palette_tree: &ImmutableKdTree<f32, 3>,
//         _: &[Vec3],
//     ) -> u8 {
//         palette_tree
//             .approx_nearest_one::<SquaredEuclidean>(&color.into())
//             .item as usize as u8
//     }
// }
//
// pub(crate) enum OrderedAlg {}
//
// impl<const MAP_SIZE: usize> Algorithm<MAP_SIZE> for OrderedAlg {
//     fn compute(
//         color: Vec3,
//         threshold: Vec3,
//         threshold_index: usize,
//         _: &mut [usize; MAP_SIZE],
//         palette_tree: &ImmutableKdTree<f32, 3>,
//         _: &[Vec3],
//     ) -> u8 {
//         palette_tree
//             .approx_nearest_one::<SquaredEuclidean>(
//                 &(color + threshold * (threshold_index as f32 / MAP_SIZE as f32 - 0.5)).into(),
//             )
//             .item as u8
//     }
// }
//
// pub(crate) enum PatternAlg {}
//
// impl<const MAP_SIZE: usize> Algorithm<MAP_SIZE> for PatternAlg {
//     fn compute(
//         color: Vec3,
//         threshold: Vec3,
//         threshold_index: usize,
//         candidates: &mut [usize; MAP_SIZE],
//         palette_tree: &ImmutableKdTree<f32, 3>,
//         palette: &[Vec3],
//     ) -> u8 {
//         let mut error = Vec3::ZERO;
//         for candidate_ref in &mut *candidates {
//             let sample = color + error * threshold;
//             let candidate = palette_tree
//                 .approx_nearest_one::<SquaredEuclidean>(&sample.into())
//                 .item as usize;
//
//             *candidate_ref = candidate;
//             error += color - palette[candidate];
//         }
//
//         candidates.sort_unstable_by(|&candidate_1, &candidate_2| {
//             palette[candidate_1][0].total_cmp(&palette[candidate_2][0])
//         });
//
//         candidates[threshold_index] as u8
//     }
// }
//
// pub(crate) fn dither_slice<A: Algorithm<MAP_SIZE>, const MAP_SIZE: usize>(
//     pixels: &mut [(usize, (&[u8], &mut Option<u8>))],
//     threshold: f32,
//     size: UVec2,
//     palette_tree: &ImmutableKdTree<f32, 3>,
//     palette: &[Vec3],
// ) where
//     (): MapSize<MAP_SIZE>,
// {
//     let mut candidates = [0; MAP_SIZE];
//
//     for &mut (i, (color, ref mut pixel)) in pixels {
//         let i = i as u32;
//         let pos = UVec2::new(i % size.x, i / size.x);
//
//         if color[3] == 0 {
//             **pixel = None;
//             continue;
//         }
//
//         **pixel = Some(A::compute(
//             Oklaba::from(Srgba::rgb_u8(color[0], color[1], color[2])).to_vec3(),
//             Vec3::splat(threshold),
//             <() as MapSize<MAP_SIZE>>::MAP[pos.x as usize % <() as MapSize<MAP_SIZE>>::WIDTH
//                 * <() as MapSize<MAP_SIZE>>::WIDTH
//                 + pos.y as usize % <() as MapSize<MAP_SIZE>>::WIDTH],
//             &mut candidates,
//             palette_tree,
//             palette,
//         ));
//     }
// }

pub(crate) type SpriteComponents<L> = (
    &'static PxSprite,
    &'static PxPosition,
    &'static PxAnchor,
    &'static L,
    &'static PxCanvas,
    Option<&'static PxFrame>,
    Option<&'static PxFilter>,
);

fn extract_sprites<L: PxLayer>(
    // TODO Maybe calculate `ViewVisibility`
    sprites: Extract<Query<(SpriteComponents<L>, &InheritedVisibility, RenderEntity)>>,
    mut cmd: Commands,
) {
    for ((sprite, &position, &anchor, layer, &canvas, frame, filter), visibility, id) in &sprites {
        let mut entity = cmd.entity(id);

        if !visibility.get() {
            // TODO Need to a better way to prevent entities from rendering
            entity.remove::<L>();
            continue;
        }

        entity.insert((sprite.clone(), position, anchor, layer.clone(), canvas));

        if let Some(frame) = frame {
            entity.insert(*frame);
        } else {
            entity.remove::<PxFrame>();
        }

        if let Some(filter) = filter {
            entity.insert(filter.clone());
        } else {
            entity.remove::<PxFilter>();
        }
    }
}

// pub(crate) type ImageToSpriteComponents<L> = (
//     &'static ImageToSprite,
//     &'static PxPosition,
//     &'static PxAnchor,
//     &'static L,
//     &'static PxCanvas,
//     Option<&'static Handle<PxFilter>>,
// );
//
// fn extract_image_to_sprites<L: PxLayer>(
//     image_to_sprites: Extract<Query<(ImageToSpriteComponents<L>, &InheritedVisibility)>>,
//     mut cmd: Commands,
// ) {
//     for ((image_to_sprite, &position, &anchor, layer, &canvas, filter), visibility) in
//         &image_to_sprites
//     {
//         if !visibility.get() {
//             continue;
//         }
//
//         let mut image_to_sprite = cmd.spawn((
//             image_to_sprite.clone(),
//             position,
//             anchor,
//             layer.clone(),
//             canvas,
//         ));
//
//         if let Some(filter) = filter {
//             image_to_sprite.insert(filter.clone());
//         }
//     }
// }

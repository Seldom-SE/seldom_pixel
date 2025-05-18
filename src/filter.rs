//! Filtering

use std::{error::Error, ops::RangeInclusive};

use bevy_asset::{io::Reader, weak_handle, AssetLoader, LoadContext};
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{component::HookContext, world::DeferredWorld};
use bevy_image::{CompressedImageFormats, ImageLoader, ImageLoaderSettings};
use bevy_math::uvec2;
use bevy_render::{
    render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin},
    sync_component::SyncComponentPlugin,
    sync_world::RenderEntity,
    Extract, RenderApp,
};

use crate::{
    animation::{draw_frame, AnimatedAssetComponent, Frames},
    image::{PxImage, PxImageSliceMut},
    palette::asset_palette,
    position::PxLayer,
    prelude::*,
};

pub const TRANSPARENT_FILTER: Handle<PxFilterAsset> =
    weak_handle!("798C57A4-A83C-5DD6-8FA6-1426E31A84CA");

pub(crate) fn plug<L: PxLayer>(app: &mut App) {
    // R-A workaround
    Assets::insert(
        &mut app
            .add_plugins((
                RenderAssetPlugin::<PxFilterAsset>::default(),
                SyncComponentPlugin::<PxFilterLayers<L>>::default(),
            ))
            .init_asset::<PxFilterAsset>()
            .init_asset_loader::<PxFilterLoader>()
            .insert_resource(InsertDefaultPxFilterLayers::new::<L>())
            .world_mut()
            .resource_mut::<Assets<PxFilterAsset>>(),
        TRANSPARENT_FILTER.id(),
        PxFilterAsset(PxImage::empty(uvec2(16, 16))),
    );

    app.sub_app_mut(RenderApp)
        .insert_resource(InsertDefaultPxFilterLayers::new::<L>())
        .add_systems(ExtractSchedule, extract_filters::<L>);
}

#[derive(Default)]
struct PxFilterLoader;

impl AssetLoader for PxFilterLoader {
    type Asset = PxFilterAsset;
    type Settings = ImageLoaderSettings;
    type Error = Box<dyn Error + Send + Sync>;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &ImageLoaderSettings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<PxFilterAsset, Self::Error> {
        let image = ImageLoader::new(CompressedImageFormats::NONE)
            .load(reader, settings, load_context)
            .await?;
        let palette = asset_palette().await;
        let indices = PxImage::palette_indices(palette, &image).map_err(|err| err.to_string())?;

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

            let index = indices.pixel(
                (UVec2::new(
                    frame_index % frame_filter_width,
                    frame_index / frame_filter_width,
                ) * frame_size
                    + UVec2::new(frame_pos % frame_size.x, frame_pos / frame_size.x))
                .as_ivec2(),
            );

            if index == 0 {
                frame_visible = true;
            }

            filter.push(index);
        }

        Ok(PxFilterAsset(PxImage::new(filter, frame_area as usize)))
    }

    fn extensions(&self) -> &[&str] {
        &["px_filter.png"]
    }
}

/// Maps colors of an image to different colors. Filter a single sprite, text, or tilemap
/// by adding a [`PxFilter`] to it, or filter entire layers
/// by spawning a [`PxFilterLayers`]. Create a [`Handle<PxFilterAsset>`] with a
/// [`PxAssets<PxFilter>`]
/// and an image file. The image should have pixels in the same positions as the palette.
/// The position of each pixel describes the mapping of colors. The image must only contain colors
/// that are also in the palette. For animated filters, arrange a number of filters
/// from the top-left corner, moving rightwards, wrapping downwards when it gets to the edge
/// of the image. For examples, see the `assets/` directory in this repository. `fade_to_black.png`
/// is an animated filter.
#[derive(Asset, Clone, Reflect, Debug)]
pub struct PxFilterAsset(pub(crate) PxImage);

impl RenderAsset for PxFilterAsset {
    type SourceAsset = Self;
    type Param = ();

    fn prepare_asset(
        source_asset: Self,
        _: AssetId<Self>,
        &mut (): &mut (),
    ) -> Result<Self, PrepareAssetError<Self>> {
        Ok(source_asset)
    }
}

impl Frames for PxFilterAsset {
    type Param = ();

    fn frame_count(&self) -> usize {
        let Self(filter) = self;
        filter.area() / filter.width()
    }

    fn draw(
        &self,
        (): (),
        image: &mut PxImageSliceMut,
        frame: impl Fn(UVec2) -> usize,
        _: impl Fn(u8) -> u8,
    ) {
        let Self(filter) = self;
        let width = image.width();
        image.for_each_mut(|index, _, pixel| {
            let index = index as u32;
            *pixel = filter.pixel(IVec2::new(
                *pixel as i32,
                frame(UVec2::new(index % width, index / width)) as i32,
            ));
        })
    }
}

impl PxFilterAsset {
    pub(crate) fn as_fn(&self) -> impl '_ + Fn(u8) -> u8 {
        let Self(filter) = self;
        |pixel| filter.pixel(IVec2::new(pixel as i32, 0))
    }
}

/// Applies a [`PxFilterAsset`] to the entity
#[derive(Component, Deref, DerefMut, Default, Clone, Debug)]
pub struct PxFilter(pub Handle<PxFilterAsset>);

impl AnimatedAssetComponent for PxFilter {
    type Asset = PxFilterAsset;

    fn handle(&self) -> &Handle<PxFilterAsset> {
        self
    }

    fn max_frame_count(asset: &PxFilterAsset) -> usize {
        asset.frame_count()
    }
}

/// Determines which layers a filter appies to
#[derive(Component, Clone)]
#[require(PxFilter, Visibility)]
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
    Range(RangeInclusive<L>),
    /// Filter applies to a set list of layers
    Many(Vec<L>),
}

impl<L: PxLayer> Default for PxFilterLayers<L> {
    fn default() -> Self {
        Self::single_clip(default())
    }
}

impl<L: PxLayer> From<RangeInclusive<L>> for PxFilterLayers<L> {
    fn from(range: RangeInclusive<L>) -> Self {
        Self::Range(range)
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

#[derive(Resource, Deref)]
struct InsertDefaultPxFilterLayers(Box<dyn Fn(bool, &mut EntityWorldMut) + Send + Sync>);

impl InsertDefaultPxFilterLayers {
    fn new<L: PxLayer>() -> Self {
        Self(Box::new(|clip, entity| {
            entity.insert_if_new(PxFilterLayers::Single {
                layer: L::default(),
                clip,
            });
        }))
    }
}

fn insert_default_px_filter_layers(mut world: DeferredWorld, ctx: HookContext) {
    world.commands().queue(move |world: &mut World| {
        let insert_default_px_filter_layers = world
            .remove_resource::<InsertDefaultPxFilterLayers>()
            .unwrap();
        if let Ok(mut entity) = world.get_entity_mut(ctx.entity) {
            if let Some(default) = entity.get::<DefaultPxFilterLayers>() {
                insert_default_px_filter_layers(
                    default.clip,
                    entity.remove::<DefaultPxFilterLayers>(),
                );
            }
        }
        world.insert_resource(insert_default_px_filter_layers);
    })
}

#[derive(Component)]
#[component(on_add = insert_default_px_filter_layers)]
pub(crate) struct DefaultPxFilterLayers {
    pub(crate) clip: bool,
}

impl Default for DefaultPxFilterLayers {
    fn default() -> Self {
        Self { clip: true }
    }
}

#[derive(Component, Default)]
pub struct PxInvertMask;

pub(crate) type FilterComponents<L> = (
    &'static PxFilter,
    &'static PxFilterLayers<L>,
    Option<&'static PxFrame>,
);

fn extract_filters<L: PxLayer>(
    filters: Extract<
        Query<(FilterComponents<L>, &InheritedVisibility, RenderEntity), Without<PxCanvas>>,
    >,
    mut cmd: Commands,
) {
    for ((filter, layers, frame), visibility, id) in &filters {
        let mut entity = cmd.entity(id);

        if !visibility.get() {
            entity.remove::<PxFilterLayers<L>>();
            continue;
        }

        entity.insert((filter.clone(), layers.clone()));

        if let Some(frame) = frame {
            entity.insert(*frame);
        } else {
            entity.remove::<PxFrame>();
        }
    }
}

pub(crate) fn draw_filter(
    filter: &PxFilterAsset,
    frame: Option<PxFrame>,
    image: &mut PxImageSliceMut,
) {
    draw_frame(filter, (), image, frame, []);
}

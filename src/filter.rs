//! Filtering

use std::time::Duration;

use anyhow::{Error, Result};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    ecs::{component::ComponentId, world::DeferredWorld},
    image::{CompressedImageFormats, ImageLoader, ImageLoaderSettings},
    render::{
        render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin},
        sync_component::SyncComponentPlugin,
        sync_world::RenderEntity,
        Extract, RenderApp,
    },
};

use crate::{
    animation::{draw_animation, AnimatedAssetComponent, Animation, PxAnimation},
    image::{PxImage, PxImageSliceMut},
    palette::asset_palette,
    pixel::Pixel,
    position::PxLayer,
    prelude::*,
};

pub(crate) fn plug<L: PxLayer>(app: &mut App) {
    app.add_plugins((
        RenderAssetPlugin::<PxFilterAsset>::default(),
        SyncComponentPlugin::<PxFilterLayers<L>>::default(),
    ))
    .init_asset::<PxFilterAsset>()
    .init_asset_loader::<PxFilterLoader>()
    .insert_resource(InsertDefaultPxFilterLayers::new::<L>())
    .sub_app_mut(RenderApp)
    .insert_resource(InsertDefaultPxFilterLayers::new::<L>())
    .add_systems(ExtractSchedule, extract_filters::<L>);
}

#[derive(Default)]
struct PxFilterLoader;

impl AssetLoader for PxFilterLoader {
    type Asset = PxFilterAsset;
    type Settings = ImageLoaderSettings;
    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &ImageLoaderSettings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<PxFilterAsset> {
        let image = ImageLoader::new(CompressedImageFormats::NONE)
            .load(reader, settings, load_context)
            .await?;
        let palette = asset_palette().await;
        let indices = PxImage::palette_indices(palette, &image)?;

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
pub struct PxFilterAsset(pub(crate) PxImage<u8>);

impl RenderAsset for PxFilterAsset {
    type SourceAsset = Self;
    type Param = ();

    fn prepare_asset(
        source_asset: Self,
        &mut (): &mut (),
    ) -> Result<Self, PrepareAssetError<Self>> {
        Ok(source_asset)
    }
}

impl Animation for PxFilterAsset {
    type Param = ();

    fn frame_count(&self) -> usize {
        let Self(filter) = self;
        filter.area() / filter.width()
    }

    fn draw(
        &self,
        (): (),
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

    fn handle(&self) -> &Handle<Self::Asset> {
        self
    }

    fn max_frame_count(asset: &PxFilterAsset) -> usize {
        asset.frame_count()
    }
}

/// Function that can be used as a layer selection function in `PxFilterLayers`. Automatically
/// implemented for types with the bounds and `Clone`.
pub trait SelectLayerFn<L: PxLayer>: 'static + Fn(&L) -> bool + Send + Sync {
    /// Clones into a trait object
    fn clone(&self) -> Box<dyn SelectLayerFn<L>>;
}

impl<L: PxLayer, T: 'static + Fn(&L) -> bool + Clone + Send + Sync> SelectLayerFn<L> for T {
    fn clone(&self) -> Box<dyn SelectLayerFn<L>> {
        Box::new(Clone::clone(self))
    }
}

impl<L: PxLayer> Clone for Box<dyn SelectLayerFn<L>> {
    fn clone(&self) -> Self {
        SelectLayerFn::clone(&**self)
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
    /// Filter applies to a set list of layers
    Many(Vec<L>),
    /// Filter applies to layers selected by the given function
    Select(Box<dyn SelectLayerFn<L>>),
}

impl<L: PxLayer> Default for PxFilterLayers<L> {
    fn default() -> Self {
        Self::single_clip(default())
    }
}

impl<L: PxLayer, T: 'static + Fn(&L) -> bool + Clone + Send + Sync> From<T> for PxFilterLayers<L> {
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

#[derive(Resource, Deref)]
struct InsertDefaultPxFilterLayers(Box<dyn Fn(&mut EntityWorldMut) + Send + Sync>);

impl InsertDefaultPxFilterLayers {
    fn new<L: PxLayer>() -> Self {
        Self(Box::new(|entity| {
            entity.insert_if_new(PxFilterLayers::<L>::default());
        }))
    }
}

#[derive(Component, Default)]
#[component(on_add = insert_default_px_filter_layers)]
pub(crate) struct DefaultPxFilterLayers;

fn insert_default_px_filter_layers(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
    world.commands().queue(move |world: &mut World| {
        let insert_default_px_filter_layers = world
            .remove_resource::<InsertDefaultPxFilterLayers>()
            .unwrap();
        if let Ok(mut entity) = world.get_entity_mut(entity) {
            insert_default_px_filter_layers(entity.remove::<DefaultPxFilterLayers>());
        }
        world.insert_resource(insert_default_px_filter_layers);
    })
}

pub(crate) type FilterComponents<L> = (
    &'static PxFilter,
    &'static PxFilterLayers<L>,
    Option<&'static PxAnimation>,
);

fn extract_filters<L: PxLayer>(
    filters: Extract<
        Query<(FilterComponents<L>, &InheritedVisibility, RenderEntity), Without<PxCanvas>>,
    >,
    mut cmd: Commands,
) {
    for ((filter, layers, animation), visibility, id) in &filters {
        let mut entity = cmd.entity(id);

        if !visibility.get() {
            entity.remove::<PxFilterLayers<L>>();
            continue;
        }

        entity.insert((filter.clone(), layers.clone()));

        if let Some(animation) = animation {
            entity.insert(*animation);
        } else {
            entity.remove::<PxAnimation>();
        }
    }
}

pub(crate) fn draw_filter(
    filter: &PxFilterAsset,
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

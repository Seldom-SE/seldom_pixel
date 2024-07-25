use anyhow::{anyhow, Error, Result};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    render::{
        render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin, RenderAssetUsages},
        texture::{ImageLoader, ImageLoaderSettings},
    },
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use crate::{
    animation::AnimationAsset, image::PxImage, palette::asset_palette, position::PxLayer,
    prelude::*,
};

pub(crate) fn plug(app: &mut App) {
    app.add_plugins(RenderAssetPlugin::<PxTypeface>::default())
        .init_asset::<PxTypeface>()
        .init_asset_loader::<PxTypefaceLoader>();
}

/// Text to be drawn on the screen
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct PxText(pub String);

impl<T: Into<String>> From<T> for PxText {
    fn from(t: T) -> Self {
        Self(t.into())
    }
}

#[derive(Serialize, Deserialize)]
struct PxTypefaceLoaderSettings {
    default_frames: u32,
    characters: String,
    character_frames: HashMap<char, u32>,
    separator_widths: HashMap<char, u32>,
    image_loader_settings: ImageLoaderSettings,
}

impl Default for PxTypefaceLoaderSettings {
    fn default() -> Self {
        Self {
            default_frames: 1,
            characters: String::new(),
            character_frames: HashMap::new(),
            separator_widths: HashMap::new(),
            image_loader_settings: default(),
        }
    }
}

struct PxTypefaceLoader(ImageLoader);

impl FromWorld for PxTypefaceLoader {
    fn from_world(world: &mut World) -> Self {
        Self(ImageLoader::from_world(world))
    }
}

impl AssetLoader for PxTypefaceLoader {
    type Asset = PxTypeface;
    type Settings = PxTypefaceLoaderSettings;
    type Error = Error;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        settings: &'a PxTypefaceLoaderSettings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<PxTypeface> {
        let Self(image_loader) = self;
        let image = image_loader
            .load(reader, &settings.image_loader_settings, load_context)
            .await?;
        let palette = asset_palette().await;
        let indices = PxImage::palette_indices(palette, &image)?;
        let height = indices.height();
        let character_count = settings.characters.chars().count();

        let characters = if character_count == 0 {
            HashMap::new()
        } else {
            settings
                .characters
                .chars()
                .zip(indices.split_vert(height / character_count).into_iter())
                .map(|(character, mut image)| {
                    image.trim_right();
                    let image_width = image.width();
                    let image_area = image.area();
                    let frames = settings
                        .character_frames
                        .get(&character)
                        .copied()
                        .unwrap_or(settings.default_frames)
                        as usize;

                    (
                        character,
                        PxSprite {
                            data: PxImage::from_parts_vert(image.split_horz(image_width / frames))
                                .unwrap(),
                            frame_size: image_area / frames,
                        },
                    )
                })
                .collect::<HashMap<_, _>>()
        };

        let max_frame_count =
            characters
                .values()
                .fold(0, |max, character| match character.frame_size > max {
                    true => character.frame_size,
                    false => max,
                });

        Ok(PxTypeface {
            height: if image.texture_descriptor.size.height == 0 {
                0
            } else if settings.characters.is_empty() {
                return Err(anyhow!(
                    "Typeface `{}` was assigned no characters. \
                        If no `.meta` file exists for that asset, create one. \
                        See `assets/typeface/` for examples.",
                    load_context.path().display()
                ));
            } else {
                image.texture_descriptor.size.height / character_count as u32
            },
            characters,
            separators: settings
                .separator_widths
                .iter()
                .map(|(&separator, &width)| (separator, PxSeparator { width }))
                .collect(),
            max_frame_count,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["px_typeface.png"]
    }
}

#[derive(Clone, Debug, Reflect)]
pub(crate) struct PxSeparator {
    pub(crate) width: u32,
}

/// A typeface. Create a [`Handle<PxTypeface>`] with a [`PxAssets<PxTypeface>`]
/// and an image file. The image file contains a column of characters, ordered from bottom to top.
/// For animated typefaces, add additional frames to the right of characters, marking the end
/// of an animation with a fully transparent character or the end of the image.
/// See the images in `assets/typeface/` for examples.
#[derive(Asset, Clone, Reflect, Debug)]
pub struct PxTypeface {
    pub(crate) height: u32,
    pub(crate) characters: HashMap<char, PxSprite>,
    pub(crate) separators: HashMap<char, PxSeparator>,
    pub(crate) max_frame_count: usize,
}

impl RenderAsset for PxTypeface {
    type SourceAsset = Self;
    type Param = ();

    fn asset_usage(_: &Self) -> RenderAssetUsages {
        RenderAssetUsages::RENDER_WORLD
    }

    fn prepare_asset(
        source_asset: Self,
        &mut (): &mut (),
    ) -> Result<Self, PrepareAssetError<Self>> {
        Ok(source_asset)
    }
}

impl AnimationAsset for PxTypeface {
    fn max_frame_count(&self) -> usize {
        self.max_frame_count
    }
}

/// Spawns text to be rendered on-screen
#[derive(Bundle, Debug, Default)]
pub struct PxTextBundle<L: PxLayer> {
    /// A [`PxText`] component
    pub text: PxText,
    /// A [`Handle<PxTypeface>`] component
    pub typeface: Handle<PxTypeface>,
    /// A [`PxRect`] component
    pub rect: PxRect,
    /// A [`PxAnchor`] component
    pub alignment: PxAnchor,
    /// A layer component
    pub layer: L,
    /// A [`PxCanvas`] component
    pub canvas: PxCanvas,
    /// A [`Visibility`] component
    pub visibility: Visibility,
}

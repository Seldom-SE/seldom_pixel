use anyhow::{anyhow, Error, Result};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    image::{CompressedImageFormats, ImageLoader, ImageLoaderSettings},
    render::{
        render_asset::{PrepareAssetError, RenderAsset, RenderAssetPlugin},
        sync_component::SyncComponentPlugin,
        sync_world::RenderEntity,
        Extract, RenderApp,
    },
    utils::HashMap,
};
use serde::{Deserialize, Serialize};

use crate::{
    animation::AnimatedAssetComponent, image::PxImage, palette::asset_palette,
    position::DefaultLayer, position::PxLayer, prelude::*,
};

pub(crate) fn plug<L: PxLayer>(app: &mut App) {
    app.add_plugins((
        RenderAssetPlugin::<PxTypeface>::default(),
        SyncComponentPlugin::<PxText>::default(),
    ))
    .init_asset::<PxTypeface>()
    .init_asset_loader::<PxTypefaceLoader>()
    .sub_app_mut(RenderApp)
    .add_systems(ExtractSchedule, extract_texts::<L>);
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

#[derive(Default)]
struct PxTypefaceLoader;

impl AssetLoader for PxTypefaceLoader {
    type Asset = PxTypeface;
    type Settings = PxTypefaceLoaderSettings;
    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &PxTypefaceLoaderSettings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<PxTypeface> {
        let image = ImageLoader::new(CompressedImageFormats::NONE)
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
                        PxSpriteAsset {
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
    pub(crate) characters: HashMap<char, PxSpriteAsset>,
    pub(crate) separators: HashMap<char, PxSeparator>,
    pub(crate) max_frame_count: usize,
}

impl PxTypeface {
    /// Check whether the typeface contains the given character, including separators
    pub fn contains(&self, character: char) -> bool {
        self.characters.contains_key(&character) || self.separators.contains_key(&character)
    }
}

impl RenderAsset for PxTypeface {
    type SourceAsset = Self;
    type Param = ();

    fn prepare_asset(
        source_asset: Self,
        &mut (): &mut (),
    ) -> Result<Self, PrepareAssetError<Self>> {
        Ok(source_asset)
    }
}

/// Spawns text to be rendered on-screen
#[derive(Component, Default, Clone, Debug, Reflect)]
#[require(PxPosition, PxAnchor, DefaultLayer, PxCanvas, Visibility)]
pub struct PxText {
    /// The contents of the text
    pub value: String,
    /// The typeface
    pub typeface: Handle<PxTypeface>,
    pub line_breaks: Vec<u32>,
}

impl PxText {
    pub fn new(value: impl Into<String>, typeface: Handle<PxTypeface>) -> Self {
        Self {
            value: value.into(),
            typeface,
            line_breaks: Vec::new(),
        }
    }
}

impl AnimatedAssetComponent for PxText {
    type Asset = PxTypeface;

    fn handle(&self) -> &Handle<Self::Asset> {
        &self.typeface
    }

    fn max_frame_count(typeface: &PxTypeface) -> usize {
        typeface.max_frame_count
    }
}

pub(crate) type TextComponents<L> = (
    &'static PxText,
    &'static PxPosition,
    &'static PxAnchor,
    &'static L,
    &'static PxCanvas,
    Option<&'static PxAnimation>,
    Option<&'static PxFilter>,
);

fn extract_texts<L: PxLayer>(
    texts: Extract<Query<(TextComponents<L>, &InheritedVisibility, RenderEntity)>>,
    mut cmd: Commands,
) {
    for ((text, &pos, &alignment, layer, &canvas, animation, filter), visibility, id) in &texts {
        let mut entity = cmd.entity(id);

        if !visibility.get() {
            entity.remove::<L>();
            continue;
        }

        entity.insert((text.clone(), pos, alignment, layer.clone(), canvas));

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

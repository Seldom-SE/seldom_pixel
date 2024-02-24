use bevy::utils::HashMap;

use crate::{
    animation::AnimationAsset,
    asset::{PxAsset, PxAssetData},
    image::PxImage,
    palette::Palette,
    position::PxLayer,
    prelude::*,
    sprite::PxSpriteData,
};

/// Text to be drawn on the screen
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct PxText(pub String);

impl<T: Into<String>> From<T> for PxText {
    fn from(t: T) -> Self {
        Self(t.into())
    }
}

/// Configuration for a character in a typeface
#[derive(Debug)]
pub struct PxCharacterConfig {
    /// The character
    pub character: char,
    /// The number of frames of animation this character has
    pub frames: u32,
}

impl From<(char, u32)> for PxCharacterConfig {
    fn from((character, frames): (char, u32)) -> Self {
        Self { character, frames }
    }
}

/// Configuration for a separator in a typeface
#[derive(Debug)]
pub struct PxSeparatorConfig {
    /// The character
    pub character: char,
    /// Width in pixels
    pub width: u32,
}

impl From<(char, u32)> for PxSeparatorConfig {
    fn from((character, width): (char, u32)) -> Self {
        Self { character, width }
    }
}

#[derive(Debug)]
pub struct PxTypefaceConfig {
    pub(crate) characters: Vec<PxCharacterConfig>,
    pub(crate) separators: Vec<PxSeparatorConfig>,
}

#[derive(Debug, Reflect)]
pub(crate) struct PxSeparator {
    pub(crate) width: u32,
}

#[derive(Debug, Reflect)]
pub struct PxTypefaceData {
    pub(crate) height: u32,
    pub(crate) characters: HashMap<char, PxSpriteData>,
    pub(crate) separators: HashMap<char, PxSeparator>,
    pub(crate) max_frame_count: usize,
}

impl PxAssetData for PxTypefaceData {
    type Config = PxTypefaceConfig;

    fn new(palette: &Palette, image: &Image, config: &Self::Config) -> Self {
        let indices = PxImage::palette_indices_unaligned(palette, image);
        let height = indices.height();

        let characters = config
            .characters
            .iter()
            .zip(
                indices
                    .split_vert(height / config.characters.len())
                    .into_iter()
                    .rev(),
            )
            .map(|(character, image)| {
                let mut image = image.flip_vert();
                image.trim_right();
                let image_width = image.width();
                let image_area = image.area();
                (
                    character.character,
                    PxSpriteData {
                        data: PxImage::from_parts_vert(
                            image.split_horz(image_width / character.frames as usize),
                        )
                        .unwrap(),
                        frame_size: image_area / character.frames as usize,
                    },
                )
            })
            .collect::<HashMap<_, _>>();

        let max_frame_count =
            characters
                .values()
                .fold(0, |max, character| match character.frame_size > max {
                    true => character.frame_size,
                    false => max,
                });

        Self {
            height: image.texture_descriptor.size.height / config.characters.len() as u32,
            characters,
            separators: config
                .separators
                .iter()
                .map(|separator| {
                    (
                        separator.character,
                        PxSeparator {
                            width: separator.width,
                        },
                    )
                })
                .collect(),
            max_frame_count,
        }
    }
}

impl AnimationAsset for PxTypefaceData {
    fn max_frame_count(&self) -> usize {
        self.max_frame_count
    }
}

/// A typeface. Create a [`Handle<PxTypeface>`] with a [`PxAssets<PxTypeface>`]
/// and an image file. The image file contains a column of characters, ordered from bottom to top.
/// For animated typefaces, add additional frames to the right of characters, marking the end
/// of an animation with a fully transparent character or the end of the image.
/// See the images in `assets/typeface/` for examples.
pub type PxTypeface = PxAsset<PxTypefaceData>;

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

//! Color palettes

use std::path::PathBuf;

use bevy::{render::render_resource::TextureFormat, utils::HashMap};

use crate::{prelude::*, set::PxSet};

pub(crate) fn palette_plugin(palette_path: PathBuf) -> impl FnOnce(&mut App) {
    move |app| {
        app.configure_sets((
            PxSet::Unloaded.run_if(resource_exists::<LoadingPalette>()),
            PxSet::Loaded.run_if(resource_exists::<Palette>()),
        ))
        .add_startup_system(load_palette(palette_path))
        .add_system(
            init_palette
                .in_set(PxSet::Unloaded)
                .in_base_set(CoreSet::PreUpdate),
        );
    }
}

#[derive(Deref, DerefMut, Resource)]
struct LoadingPalette(Handle<Image>);

/// Resource representing the game's palette. The palette is loaded from an image containing pixels
/// that represent what colors the game may display. You may use up to 255 colors.
/// The bottom-left pixel in the palette is used as the background color. Set this resource
/// to a new palette to change the game's palette. The replacement palette's pixels
/// must be laid out the same as the original. You cannot change the palette that is used
/// to load assets.
#[derive(Clone, Debug, Resource)]
pub struct Palette {
    pub(crate) size: UVec2,
    pub(crate) colors: Vec<[u8; 3]>,
    pub(crate) indices: HashMap<[u8; 3], u8>,
}

/// Internal resource representing the palette used to load assets.
#[derive(Debug, Resource)]
pub struct AssetPalette(pub(crate) Palette);

impl Palette {
    /// Create a palette from an [`Image`]
    pub fn new(palette: &Image) -> Palette {
        let colors = palette
            .convert(TextureFormat::Rgba8UnormSrgb)
            .unwrap()
            .data
            .chunks_exact(palette.texture_descriptor.size.width as usize * 4)
            .rev()
            .flatten()
            .copied()
            .fold(
                (Vec::default(), [0, 0, 0], 0),
                |(mut colors, mut color, i), value| {
                    if i == 3 {
                        if value != 0 {
                            colors.push(color);
                        }
                        (colors, [0, 0, 0], 0)
                    } else {
                        color[i] = value;
                        (colors, color, i + 1)
                    }
                },
            )
            .0;

        Palette {
            size: UVec2::new(
                palette.texture_descriptor.size.width,
                palette.texture_descriptor.size.height,
            ),
            indices: colors
                .iter()
                .enumerate()
                .map(|(i, color)| (*color, i as u8))
                .collect(),
            colors,
        }
    }
}

fn load_palette(path: PathBuf) -> impl Fn(Commands, Res<AssetServer>) {
    move |mut commands, assets| {
        commands.insert_resource(LoadingPalette(assets.load(path.clone())));
    }
}

fn init_palette(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    loading_palette: Res<LoadingPalette>,
) {
    if let Some(palette) = images.get(&**loading_palette) {
        let palette = Palette::new(palette);
        commands.insert_resource(palette.clone());
        commands.insert_resource(AssetPalette(palette));
        commands.remove_resource::<LoadingPalette>();
    }
}

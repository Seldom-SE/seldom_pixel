//! Color palettes

use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

use anyhow::{Error, Result};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    render::{
        render_resource::TextureFormat,
        texture::{ImageLoader, ImageLoaderSettings},
    },
    utils::HashMap,
};
use event_listener::Event;
use seldom_singleton::AssetSingleton;

use crate::prelude::*;

pub(crate) fn plug(palette_path: PathBuf) -> impl Fn(&mut App) {
    move |app| {
        app.init_asset::<Palette>()
            .init_asset_loader::<PaletteLoader>()
            .add_systems(Startup, init_palette(palette_path.clone()))
            .add_systems(
                PreUpdate,
                load_asset_palette.run_if(resource_exists::<LoadingAssetPaletteHandle>),
            );
    }
}

struct PaletteLoader(ImageLoader);

impl FromWorld for PaletteLoader {
    fn from_world(world: &mut World) -> Self {
        Self(ImageLoader::from_world(world))
    }
}

impl AssetLoader for PaletteLoader {
    type Asset = Palette;
    type Settings = ImageLoaderSettings;
    type Error = Error;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        settings: &'a ImageLoaderSettings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Palette> {
        let Self(image_loader) = self;
        Ok(Palette::new(
            &image_loader.load(reader, settings, load_context).await?,
        ))
    }

    fn extensions(&self) -> &[&str] {
        &["palette.png"]
    }
}

/// A palette. Palettes are loaded from images containing pixels
/// that represent what colors the game may display. You may use up to 255 colors.
/// The bottom-left pixel in the palette is used as the background color.
#[derive(Asset, Clone, TypePath, Debug)]
pub struct Palette {
    pub(crate) size: UVec2,
    // TODO This could be a `[[u8; 3]; 255]`
    pub(crate) colors: Vec<[u8; 3]>,
    pub(crate) indices: HashMap<[u8; 3], u8>,
}

/// Resource containing the game's palette. Set this resource
/// to a new palette to change the game's palette. The replacement palette's pixels
/// must be laid out the same as the original. You cannot change the palette that is used
/// to load assets.
#[derive(Resource, Deref, DerefMut)]
pub struct PaletteHandle(pub Handle<Palette>);

pub(crate) type PaletteParam<'w> = AssetSingleton<'w, PaletteHandle>;

#[derive(Resource, Deref)]
struct LoadingAssetPaletteHandle(Handle<Palette>);

type LoadingAssetPaletteParam<'w> = AssetSingleton<'w, LoadingAssetPaletteHandle>;

impl Palette {
    /// Create a palette from an [`Image`]
    pub fn new(palette: &Image) -> Palette {
        let colors = palette
            .convert(TextureFormat::Rgba8UnormSrgb)
            .unwrap()
            .data
            .iter()
            .copied()
            // TODO Should use chunks here
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

fn init_palette(path: PathBuf) -> impl Fn(Commands, Res<AssetServer>) {
    move |mut commands, assets| {
        let palette = assets.load(path.clone());
        commands.insert_resource(PaletteHandle(palette.clone()));
        commands.insert_resource(LoadingAssetPaletteHandle(palette));
    }
}

/// # Safety
///
/// Must not be read before `ASSET_PALETTE_INITIALIZED` is set. Must not be mutated after
/// `ASSET_PALETTE_INITIALIZED` is set.
static mut ASSET_PALETTE: Option<Palette> = None;
/// Must not be unset after it has been set
static ASSET_PALETTE_INITIALIZED: AtomicBool = AtomicBool::new(false);
/// Notifies after `ASSET_PALETTE_INITIALIZED` is set
static ASSET_PALETTE_JUST_INITIALIZED: Event = Event::new();

pub(crate) async fn asset_palette() -> &'static Palette {
    if ASSET_PALETTE_INITIALIZED.load(Ordering::SeqCst) {
        // SAFETY: Checked above
        return unsafe { ASSET_PALETTE.as_ref() }.unwrap();
    }

    let just_initialized = ASSET_PALETTE_JUST_INITIALIZED.listen();

    if ASSET_PALETTE_INITIALIZED.load(Ordering::SeqCst) {
        // SAFETY: Checked above
        return unsafe { ASSET_PALETTE.as_ref() }.unwrap();
    }

    just_initialized.await;
    // SAFETY: `just_initialized` finished waiting, so `ASSET_PALETTE_INITIALIZED` is set
    return unsafe { ASSET_PALETTE.as_ref() }.unwrap();
}

fn load_asset_palette(palette: LoadingAssetPaletteParam, mut cmd: Commands) {
    let Some(palette) = palette.get() else {
        return;
    };

    if ASSET_PALETTE_INITIALIZED.load(Ordering::SeqCst) {
        panic!("Tried to set the asset palette after it was initialized");
    }

    let palette = Some(palette.clone());
    // SAFETY: Checked above
    unsafe { ASSET_PALETTE = palette };
    ASSET_PALETTE_INITIALIZED.store(true, Ordering::SeqCst);
    ASSET_PALETTE_JUST_INITIALIZED.notify(usize::MAX);

    cmd.remove_resource::<LoadingAssetPaletteHandle>();
}

//! Asset loading

use std::fmt::Debug;

use bevy::{
    asset::{Asset, AssetPath},
    ecs::system::SystemParam,
    reflect::{TypeUuid, Uuid},
};

use seldom_fn_plugin::FnPluginExt;

#[cfg(feature = "map")]
use crate::map::PxTilesetData;
use crate::{
    filter::PxFilterData,
    palette::{AssetPalette, Palette},
    prelude::*,
    set::PxSet,
    sprite::PxSpriteData,
    text::{PxCharacterConfig, PxTypefaceConfig, PxTypefaceData},
};

use self::sealed::{PxAssetDataSealed, PxAssetTraitSealed};

pub(crate) fn asset_plugin(app: &mut App) {
    app.configure_set(
        PxSet::LoadAssets
            .before(PxSet::Draw)
            .in_set(PxSet::Loaded)
            .in_base_set(CoreSet::PostUpdate),
    )
    .fn_plugin(px_asset_plugin::<PxSpriteData>)
    .fn_plugin(px_asset_plugin::<PxTypefaceData>)
    .fn_plugin(px_asset_plugin::<PxFilterData>);
    #[cfg(feature = "map")]
    app.fn_plugin(px_asset_plugin::<PxTilesetData>);
}

fn px_asset_plugin<D: PxAssetData>(app: &mut App) {
    app.add_asset::<PxAsset<D>>()
        .init_resource::<LoadingAssets<D>>()
        .add_system(D::load.in_set(PxSet::LoadAssets));
}

/// An asset created from an image
#[derive(Debug)]
pub enum PxAsset<D: PxAssetData> {
    /// Waiting for the source image to load
    Loading {
        /// Source image for this asset
        image: Handle<Image>,
    },
    /// The asset has been created
    Loaded {
        /// The loaded asset
        asset: D,
    },
}

impl<D: PxAssetData> TypeUuid for PxAsset<D> {
    const TYPE_UUID: Uuid = Uuid::from_bytes(D::UUID);
}

/// Internal trait implemented for [`PxAsset<impl PxAssetData>`]
pub trait PxAssetTrait: Asset + PxAssetTraitSealed {
    /// The data associated with this asset
    type Data: PxAssetData;
}

impl<D: PxAssetData> PxAssetTrait for PxAsset<D>
where
    PxAsset<D>: PxAssetTraitSealed,
{
    type Data = D;
}

/// Internal trait implemented for asset data types
pub trait PxAssetData: 'static + Debug + PxAssetDataSealed + Send + Sized + Sync {
    /// Uuid used to implement [`TypeUuid`]
    const UUID: [u8; 16];
    /// Additional configuration needed to create this asset
    type Config: Debug + Send + Sync;

    /// Create the asset from an image
    fn new(palette: &Palette, image: &Image, config: &Self::Config) -> Self;

    /// System to load this asset
    fn load(
        palette: Res<AssetPalette>,
        images: Res<Assets<Image>>,
        mut assets: ResMut<Assets<PxAsset<Self>>>,
        mut loading_assets: ResMut<LoadingAssets<Self>>,
    ) {
        let mut loaded = Vec::default();
        let LoadingAssets(loading_assets) = &mut *loading_assets;

        for (i, loading_asset) in loading_assets.iter().enumerate() {
            let asset = assets.get_mut(&loading_asset.handle).unwrap();
            if let Some(image) = images.get(match asset {
                PxAsset::Loading { image } => &*image,
                PxAsset::Loaded { .. } => {
                    panic!("loaded asset was found in `LoadingAssets<D>`")
                }
            }) {
                let AssetPalette(palette) = &*palette;
                *asset = PxAsset::Loaded {
                    asset: Self::new(palette, image, &loading_asset.config),
                };

                loaded.push(i);
            }
        }

        for i in loaded.into_iter().rev() {
            loading_assets.remove(i);
        }
    }
}

#[derive(Debug)]
struct LoadingAsset<D: PxAssetData> {
    config: D::Config,
    handle: Handle<PxAsset<D>>,
}

/// List of assets that are currently loading
#[derive(Debug, Resource)]
pub struct LoadingAssets<D: PxAssetData>(Vec<LoadingAsset<D>>);

impl<D: PxAssetData> Default for LoadingAssets<D> {
    fn default() -> Self {
        Self(default())
    }
}

/// System parameter used to load `seldom_pixel` assets. Only tested with `.png` images.
#[derive(SystemParam)]
pub struct PxAssets<'w, 's, A: PxAssetTrait> {
    _query: Query<'w, 's, ()>,
    asset_server: Res<'w, AssetServer>,
    assets: ResMut<'w, Assets<PxAsset<A::Data>>>,
    loading_resource: ResMut<'w, LoadingAssets<A::Data>>,
}

impl<'w, 's, A: PxAssetTrait> PxAssets<'w, 's, A> {
    fn load_internal<'a>(
        &mut self,
        path: impl Into<AssetPath<'a>>,
        config: <A::Data as PxAssetData>::Config,
    ) -> Handle<PxAsset<A::Data>> {
        let handle = self.assets.add(PxAsset::Loading {
            image: self.asset_server.load(path),
        });

        let LoadingAssets(loading_assets) = &mut *self.loading_resource;
        loading_assets.push(LoadingAsset {
            config,
            handle: handle.clone(),
        });
        handle
    }
}

impl<'w, 's> PxAssets<'w, 's, PxSprite> {
    /// Loads a sprite
    pub fn load<'a>(&mut self, path: impl Into<AssetPath<'a>>) -> Handle<PxSprite> {
        self.load_internal(path, 1)
    }

    /// Loads an animated sprite
    pub fn load_animated<'a>(
        &mut self,
        path: impl Into<AssetPath<'a>>,
        frames: usize,
    ) -> Handle<PxSprite> {
        self.load_internal(path, frames)
    }
}

#[cfg(feature = "map")]
impl<'w, 's> PxAssets<'w, 's, PxTileset> {
    /// Loads a tileset. Works for animated tilesets.
    pub fn load<'a>(
        &mut self,
        path: impl Into<AssetPath<'a>>,
        tile_size: UVec2,
    ) -> Handle<PxTileset> {
        self.load_internal(path, tile_size)
    }
}

impl<'w, 's> PxAssets<'w, 's, PxTypeface> {
    /// Loads a typeface
    pub fn load<'a>(
        &mut self,
        path: impl Into<AssetPath<'a>>,
        characters: &str,
        separators: impl IntoIterator<Item = impl Into<PxSeparatorConfig>>,
    ) -> Handle<PxTypeface> {
        self.load_internal(
            path,
            PxTypefaceConfig {
                characters: characters
                    .chars()
                    .map(|character| (character, 1).into())
                    .collect(),
                separators: separators.into_iter().map(Into::into).collect(),
            },
        )
    }

    /// Loads an animated typeface
    pub fn load_animated<'a>(
        &mut self,
        path: impl Into<AssetPath<'a>>,
        characters: impl IntoIterator<Item = impl Into<PxCharacterConfig>>,
        separators: impl IntoIterator<Item = impl Into<PxSeparatorConfig>>,
    ) -> Handle<PxTypeface> {
        self.load_internal(
            path,
            PxTypefaceConfig {
                characters: characters.into_iter().map(Into::into).collect(),
                separators: separators.into_iter().map(Into::into).collect(),
            },
        )
    }
}

impl<'w, 's> PxAssets<'w, 's, PxFilter> {
    /// Loads a filter. Works for animated filters.
    pub fn load<'a>(&mut self, path: impl Into<AssetPath<'a>>) -> Handle<PxFilter> {
        self.load_internal(path, ())
    }
}

pub(crate) fn get_asset<'a, D: PxAssetData>(
    assets: &'a Assets<PxAsset<D>>,
    handle: Option<&Handle<PxAsset<D>>>,
) -> Option<&'a D> {
    handle.and_then(|handle| {
        assets.get(handle).and_then(|asset| match asset {
            PxAsset::Loaded { asset } => Some(asset),
            _ => None,
        })
    })
}

mod sealed {
    #[cfg(feature = "map")]
    use crate::map::PxTilesetData;
    use crate::{filter::PxFilterData, prelude::*, sprite::PxSpriteData, text::PxTypefaceData};

    pub trait PxAssetTraitSealed {}

    impl PxAssetTraitSealed for PxSprite {}

    #[cfg(feature = "map")]
    impl PxAssetTraitSealed for PxTileset {}

    impl PxAssetTraitSealed for PxTypeface {}

    impl PxAssetTraitSealed for PxFilter {}

    pub trait PxAssetDataSealed {}

    impl PxAssetDataSealed for PxSpriteData {}

    #[cfg(feature = "map")]
    impl PxAssetDataSealed for PxTilesetData {}

    impl PxAssetDataSealed for PxTypefaceData {}

    impl PxAssetDataSealed for PxFilterData {}
}

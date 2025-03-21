//! Module for convenient imports. Use with `use seldom_pixel::prelude::*;`.

pub(crate) use bevy::prelude::*;
#[cfg(feature = "particle")]
pub(crate) use bevy_turborand::prelude::*;
#[cfg(feature = "nav")]
pub(crate) use seldom_map_nav::prelude::*;
#[cfg(feature = "state")]
pub(crate) use seldom_state::prelude::*;

#[cfg(feature = "line")]
pub use crate::line::PxLine;
#[cfg(feature = "particle")]
pub use crate::particle::{PxEmitter, PxEmitterFrequency, PxEmitterSimulation, PxParticleLifetime};
pub use crate::{
    animation::{
        PxAnimation, PxAnimationDirection, PxAnimationDuration, PxAnimationFinishBehavior,
        PxAnimationFinished, PxAnimationFrameTransition,
    },
    button::{PxButtonFilter, PxButtonSprite, PxClick, PxEnableButtons, PxHover, PxInteractBounds},
    camera::{PxCamera, PxCanvas},
    cursor::PxCursor,
    filter::{PxFilter, PxFilterAsset, PxFilterLayers},
    map::{PxMap, PxTile, PxTiles, PxTileset},
    math::{Diagonal, Orthogonal},
    position::{PxAnchor, PxLayer, PxPosition, PxSubPosition, PxVelocity},
    screen::ScreenSize,
    sprite::{PxSprite, PxSpriteAsset},
    text::{PxText, PxTypeface},
    ui::{
        PxContainer, PxGrid, PxGridBuilder, PxKeyField, PxKeyFieldUpdate, PxRect, PxRow,
        PxRowBuilder, PxRowSlot, PxSlotBuilder, PxSpace, PxStack, PxStackBuilder, PxTextField,
        PxTextFieldUpdate, PxUiBuilder,
    },
    PxPlugin,
};
pub use seldom_pixel_macros::px_layer;

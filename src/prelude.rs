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
    camera::{PxCamera, PxCanvas},
    cursor::PxCursor,
    filter::{PxFilter, PxFilterAsset, PxFilterLayers},
    map::{PxMap, PxTile, PxTiles, PxTileset},
    math::{Diagonal, Orthogonal},
    position::{PxAnchor, PxLayer, PxPosition, PxSubPosition, PxVelocity},
    rect::PxRect,
    screen::ScreenSize,
    sprite::{PxSprite, PxSpriteAsset},
    text::{PxText, PxTypeface},
    ui::{
        PxContainer, PxGrid, PxGridBuilder, PxKeyField, PxKeyFieldUpdate, PxRectBuilder, PxRow,
        PxRowBuilder, PxRowSlot, PxSlotBuilder, PxSpace, PxStack, PxStackBuilder, PxTextBuilder,
        PxTextField, PxTextFieldUpdate, PxUiBuilder,
    },
    PxPlugin,
};

pub use next::Next;
pub use seldom_pixel_macros::px_layer;

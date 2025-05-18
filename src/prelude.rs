//! Module for convenient imports. Use with `use seldom_pixel::prelude::*;`.

pub(crate) use bevy_app::prelude::*;
pub(crate) use bevy_asset::prelude::*;
pub(crate) use bevy_color::prelude::*;
pub(crate) use bevy_ecs::prelude::*;
pub(crate) use bevy_image::prelude::*;
pub(crate) use bevy_input::prelude::*;
pub(crate) use bevy_log::prelude::*;
pub(crate) use bevy_math::prelude::*;
pub(crate) use bevy_reflect::prelude::*;
pub(crate) use bevy_render::prelude::*;
pub(crate) use bevy_time::prelude::*;
pub(crate) use bevy_transform::prelude::*;
#[cfg(feature = "particle")]
pub(crate) use bevy_turborand::prelude::*;
pub(crate) use bevy_utils::prelude::*;
pub(crate) use bevy_window::prelude::*;
#[cfg(feature = "nav")]
pub(crate) use seldom_map_nav::prelude::*;
#[cfg(feature = "state")]
pub(crate) use seldom_state::prelude::*;

pub(crate) const OK: Result = Ok(());

#[cfg(feature = "line")]
pub use crate::line::PxLine;
#[cfg(feature = "particle")]
pub use crate::particle::{PxEmitter, PxEmitterFrequency, PxEmitterSimulation, PxParticleLifetime};
pub use crate::{
    animation::{
        PxAnimation, PxAnimationDirection, PxAnimationDuration, PxAnimationFinishBehavior,
        PxAnimationFinished, PxFrame, PxFrameSelector, PxFrameTransition,
    },
    camera::{PxCamera, PxCanvas},
    cursor::PxCursor,
    filter::{PxFilter, PxFilterAsset, PxFilterLayers, PxInvertMask},
    map::{PxMap, PxTile, PxTiles, PxTileset},
    math::{Diagonal, Orthogonal},
    position::{PxAnchor, PxLayer, PxPosition, PxSubPosition, PxVelocity},
    rect::PxRect,
    screen::ScreenSize,
    sprite::{PxSprite, PxSpriteAsset},
    text::{PxText, PxTypeface},
    ui::{
        PxContainer, PxContainerBuilder, PxGrid, PxGridBuilder, PxKeyField, PxKeyFieldBuilder,
        PxKeyFieldUpdate, PxMinSize, PxMinSizeBuilder, PxRectBuilder, PxRow, PxRowBuilder,
        PxRowSlot, PxScroll, PxScrollBuilder, PxSlotBuilder, PxSpace, PxSpriteBuilder, PxStack,
        PxStackBuilder, PxTextBuilder, PxTextField, PxTextFieldBuilder, PxTextFieldUpdate,
        PxUiBuilder,
    },
    PxPlugin,
};

pub use seldom_pixel_macros::px_layer;

//! Module for convenient imports. Use with `use seldom_pixel::prelude::*;`.

pub(crate) use bevy::prelude::*;
#[cfg(feature = "map")]
pub(crate) use bevy_ecs_tilemap::prelude::*;
pub(crate) use iyes_loopless::prelude::*;
#[cfg(feature = "interop")]
pub(crate) use seldom_interop::prelude::*;
#[cfg(feature = "state")]
pub(crate) use seldom_state::prelude::*;

#[cfg(feature = "line")]
pub use crate::line::{PxLine, PxLineBundle};
#[cfg(feature = "map")]
pub use crate::map::{PxMapBundle, PxTileBundle, PxTileset};
#[cfg(feature = "particle")]
pub use crate::particle::{
    PxEmitterBundle, PxEmitterFn, PxEmitterFrequency, PxEmitterRange, PxEmitterSimulation,
    PxEmitterSprites, PxParticleLifetime,
};
pub use crate::{
    animation::{
        PxAnimationBundle, PxAnimationDirection, PxAnimationDuration, PxAnimationFinishBehavior,
        PxAnimationFinished, PxAnimationFrameTransition,
    },
    asset::PxAssets,
    button::{
        PxButtonFilterBundle, PxButtonSpriteBundle, PxClick, PxClickFilter, PxClickSprite,
        PxEnableButtons, PxHover, PxHoverFilter, PxHoverSprite, PxIdleFilter, PxIdleSprite,
        PxInteractBounds,
    },
    camera::{PxCamera, PxCanvas},
    cursor::PxCursor,
    filter::{PxFilter, PxFilterBundle, PxFilterLayers},
    math::IRect,
    position::{PxAnchor, PxLayer, PxPosition, PxSubPosition, PxVelocity},
    px_plugin,
    sprite::{PxSprite, PxSpriteBundle},
    text::{PxCharacterConfig, PxSeparatorConfig, PxText, PxTextBundle, PxTypeface},
    PxPlugin,
};
pub use seldom_pixel_macros::px_layer;

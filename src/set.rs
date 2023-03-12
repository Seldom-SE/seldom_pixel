//! Stages used by this crate

use crate::prelude::*;

/// Stages used by this crate
#[derive(Clone, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum PxSet {
    // General
    /// Runs if the palette is not loaded
    Unloaded,
    /// Runs if the palette is loaded
    Loaded,

    // `PreUpdate`
    /// The [`PxPosition`] is updated to match [`PxSubPosition`]. In [`CoreSet::PreUpdate`].
    UpdatePosToSubPos,
    /// [`crate::cursor::PxCursorPosition`] is updated. In [`CoreSet::PreUpdate`].
    UpdateCursorPosition,

    // `PostUpdate`
    /// `seldom_pixel` assets are loaded. In [`CoreSet::PostUpdate`].
    LoadAssets,
    /// New buttons have assets added to them. In [`CoreSet::PostUpdate`].
    AddButtonAssets,
    /// Button assets are updated. In [`CoreSet::PostUpdate`].
    UpdateButtonAssets,
    /// Animations are completed. In [`CoreSet::PostUpdate`].
    FinishAnimations,
    /// Update particle emitters. In [`CoreSet::PostUpdate`].
    #[cfg(feature = "particle")]
    UpdateEmitters,
    /// The screen is drawn. In [`CoreSet::PostUpdate`].
    Draw,
    /// The cursor is drawn. In [`CoreSet::PostUpdate`].
    DrawCursor,
}

//! Sets used by this crate

use crate::prelude::*;

// TODO Many of these aren't necessary anymore
/// Sets used by this crate
#[derive(Clone, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum PxSet {
    // `PreUpdate`
    /// The [`PxPosition`] is updated to match [`PxSubPosition`]. In [`CoreSet::PreUpdate`].
    UpdatePosToSubPos,
    /// [`crate::cursor::PxCursorPosition`] is updated. In [`CoreSet::PreUpdate`].
    UpdateCursorPosition,

    // `PostUpdate`
    /// New buttons have assets added to them. In [`CoreSet::PostUpdate`].
    AddButtonAssets,
    /// Button assets are updated. In [`CoreSet::PostUpdate`].
    UpdateButtonAssets,
    /// Animations are completed. In [`CoreSet::PostUpdate`].
    FinishAnimations,
    /// Update particle emitters. In [`CoreSet::PostUpdate`].
    #[cfg(feature = "particle")]
    UpdateEmitters,
    Picking,
}

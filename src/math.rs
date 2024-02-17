//! Math helpers

use crate::prelude::*;

/// Extension crate for [`IRect`]. Adds helpers.
pub trait RectExt {
    /// Creates a rectangle from a position, size, and anchor
    fn pos_size_anchor(pos: IVec2, size: UVec2, anchor: PxAnchor) -> Self;

    /// Subtracts an [`IVec2`] from the rectangle's points
    fn sub_ivec2(self, other: IVec2) -> Self;
}

impl RectExt for IRect {
    fn pos_size_anchor(pos: IVec2, size: UVec2, anchor: PxAnchor) -> Self {
        let min = pos - anchor.pos(size).as_ivec2();

        Self {
            min,
            max: min + size.as_ivec2(),
        }
    }

    fn sub_ivec2(self, other: IVec2) -> Self {
        Self {
            min: self.min - other,
            max: self.max - other,
        }
    }
}

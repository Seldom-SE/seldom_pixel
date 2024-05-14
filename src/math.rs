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

#[derive(Copy, Clone)]
pub enum Diagonal {
    UpRight,
    UpLeft,
    DownLeft,
    DownRight,
}

impl Diagonal {
    pub fn as_uvec2(self) -> UVec2 {
        use Diagonal::*;

        match self {
            UpRight => UVec2::ONE,
            UpLeft => UVec2::new(0, 1),
            DownLeft => UVec2::ZERO,
            DownRight => UVec2::new(1, 0),
        }
    }
}

//! Math helpers

use crate::prelude::*;

/// Extension trait for [`IRect`]. Adds helpers.
pub trait RectExt {
    /// Like `contains`, but excludes the end bounds of the rectangle
    fn contains_exclusive(self, point: IVec2) -> bool;

    /// Creates a rectangle from a position, size, and anchor
    fn pos_size_anchor(pos: IVec2, size: UVec2, anchor: PxAnchor) -> Self;

    /// Subtracts an [`IVec2`] from the rectangle's points
    fn sub_ivec2(self, other: IVec2) -> Self;
}

impl RectExt for IRect {
    fn contains_exclusive(self, point: IVec2) -> bool {
        point.cmpge(self.min).all() && point.cmplt(self.max).all()
    }

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

/// An orthogonal direction
#[derive(Debug)]
pub enum Orthogonal {
    /// Right
    Right,
    /// Up
    Up,
    /// Left
    Left,
    /// Down
    Down,
}

/// A diagonal direction
#[derive(Copy, Clone)]
pub enum Diagonal {
    /// Up-right
    UpRight,
    /// Up-left
    UpLeft,
    /// Down-left
    DownLeft,
    /// Down-right
    DownRight,
}

impl Diagonal {
    /// 1 for each positive axis and 0 for each negative axis
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

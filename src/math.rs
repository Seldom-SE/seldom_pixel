//! Math helper types

use std::ops::Sub;

use crate::prelude::*;

/// Rectangle using `u32`
#[derive(Clone, Copy, Debug)]
pub struct URect {
    /// Bottom-left corner
    pub min: UVec2,
    /// Top-right corner
    pub max: UVec2,
}

impl URect {
    /// Convert to an `IRect`
    pub fn as_irect(self) -> IRect {
        IRect {
            min: self.min.as_ivec2(),
            max: self.max.as_ivec2(),
        }
    }
}

/// Rectangle using `i32`
#[derive(Clone, Copy, Debug, Default)]
pub struct IRect {
    /// Bottom-left corner
    pub min: IVec2,
    /// Top-right corner
    pub max: IVec2,
}

impl Sub<IVec2> for IRect {
    type Output = Self;

    fn sub(self, rhs: IVec2) -> Self::Output {
        Self {
            min: self.min - rhs,
            max: self.max - rhs,
        }
    }
}

impl IRect {
    /// Creates an `IRect`
    pub fn new(min: IVec2, max: IVec2) -> Self {
        Self { min, max }
    }

    /// Creates an `IRect` from a position, size, and anchor.
    pub fn pos_size_anchor(pos: IVec2, size: UVec2, anchor: PxAnchor) -> Self {
        let min = pos - anchor.pos(size).as_ivec2();
        let max = min + size.as_ivec2();

        IRect { min, max }
    }

    /// Gets the size of the rectangle
    pub fn size(self) -> UVec2 {
        (self.max - self.min).as_uvec2()
    }

    /// Gets the center of the rectangle
    pub fn center(self) -> IVec2 {
        (self.max + self.min) / 2
    }

    /// Determines whether the rectangle contains the given point
    pub fn contains(self, point: IVec2) -> bool {
        point.cmpge(self.min).all() && point.cmplt(self.max).all()
    }

    /// Determines whether this rectangle and the given rectangle intersect
    pub fn intersects(self, other: Self) -> bool {
        self.min.x < other.max.x
            && other.min.x < self.max.x
            && self.min.y < other.max.y
            && other.min.y < self.max.y
    }

    /// Finds the rectangle of intersection between this rectangle and the given rectangle.
    /// If the rectangles don't intersect, the resulting rectangle will be invalid
    pub fn intersection(self, other: Self) -> Self {
        let x1 = self.min.x.max(other.min.x);
        let y1 = self.min.y.max(other.min.y);
        let x2 = self.max.x.min(other.max.x);
        let y2 = self.max.y.min(other.max.y);

        Self {
            min: IVec2::new(x1.min(x2), y1.min(y2)),
            max: IVec2::new(x1.max(x2), y1.max(y2)),
        }
    }
}

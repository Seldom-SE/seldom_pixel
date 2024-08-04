use crate::{position::Spatial, prelude::*};

/// UI is displayed within these bounds
#[derive(Component, Deref, DerefMut, Clone, Copy, Default, Debug)]
pub struct PxRect(pub IRect);

impl From<IRect> for PxRect {
    fn from(rect: IRect) -> Self {
        Self(rect)
    }
}

impl Spatial for PxRect {
    fn frame_size(&self) -> UVec2 {
        self.size().as_uvec2()
    }
}

/// Cross-axis alignment
#[derive(Debug, Default)]
pub enum Align {
    /// Align to the start of the space
    #[default]
    Start,
    /// Align to the end of the space
    End,
}

/// Lays out a spatial entity's children in a direction
#[derive(Component, Debug)]
pub struct PxLayout {
    /// Direction in which to lay out the children
    pub direction: Orthogonal,
    /// Cross-axis alignment
    pub align: Align,
    /// Space between each child
    pub spacing: u16,
}

impl Default for PxLayout {
    fn default() -> Self {
        Self {
            direction: Orthogonal::Right,
            align: default(),
            spacing: 0,
        }
    }
}

/// Lays out its children in a direction
#[derive(Bundle, Default, Debug)]
pub struct PxLayoutBundle {
    /// A `PxLayout` component
    pub layout: PxLayout,
    /// A `PxRect` component
    pub rect: PxRect,
    /// A `PxPosition` component
    pub position: PxPosition,
}

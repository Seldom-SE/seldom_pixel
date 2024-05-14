use crate::{position::Spatial, prelude::*};

/// UI is displayed within these bounds
#[derive(Component, Debug, Default, Deref, DerefMut)]
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

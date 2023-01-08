use crate::prelude::*;

/// UI is displayed within these bounds
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct PxRect(pub IRect);

impl From<IRect> for PxRect {
    fn from(rect: IRect) -> Self {
        Self(rect)
    }
}

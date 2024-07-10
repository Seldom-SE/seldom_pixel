use crate::prelude::*;

pub(crate) fn plug(app: &mut App) {
    app.init_resource::<PxCamera>();
}

/// Resource that represents the camera's position
#[derive(Clone, Copy, Debug, Default, Deref, DerefMut, Resource)]
pub struct PxCamera(pub IVec2);

/// Determines whether the entity is locked to the camera
#[derive(Clone, Component, Copy, Debug, Default)]
pub enum PxCanvas {
    /// The entity is drawn relative to the world, like terrain
    #[default]
    World,
    /// The entity is drawn relative to the camera, like UI
    Camera,
}

use bevy_derive::{Deref, DerefMut};
#[cfg(feature = "headed")]
use bevy_render::{
    extract_component::ExtractComponent,
    extract_resource::{ExtractResource, ExtractResourcePlugin},
};

use crate::prelude::*;

pub(crate) fn plug(app: &mut App) {
    #[cfg(feature = "headed")]
    app.add_plugins(ExtractResourcePlugin::<PxCamera>::default());
    app.init_resource::<PxCamera>();
}

/// Resource that represents the camera's position
#[cfg_attr(feature = "headed", derive(ExtractResource))]
#[derive(Resource, Deref, DerefMut, Clone, Copy, Default, Debug)]
pub struct PxCamera(pub IVec2);

/// Determines whether the entity is locked to the camera
#[cfg_attr(feature = "headed", derive(ExtractComponent))]
#[derive(Component, Clone, Copy, Default, Reflect, Debug)]
pub enum PxCanvas {
    /// The entity is drawn relative to the world, like terrain
    #[default]
    World,
    /// The entity is drawn relative to the camera, like UI
    Camera,
}

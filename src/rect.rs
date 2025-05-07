use bevy::render::{sync_world::RenderEntity, Extract, RenderApp};

use crate::{
    animation::Animation, filter::DefaultPxFilterLayers, image::PxImageSliceMut, pixel::Pixel,
    position::Spatial, prelude::*,
};

pub(crate) fn plug<L: PxLayer>(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .add_systems(ExtractSchedule, extract_rects::<L>);
}

/// A rectangle in which a filter is applied
#[derive(Component, Deref, DerefMut, Clone, Copy, Reflect)]
#[require(
    PxFilter,
    DefaultPxFilterLayers,
    PxPosition,
    PxAnchor,
    PxCanvas,
    Visibility
)]
pub struct PxRect(pub UVec2);

impl Animation for (PxRect, &PxFilterAsset) {
    type Param = ();

    fn frame_count(&self) -> usize {
        self.1.frame_count()
    }

    fn draw(
        &self,
        (): (),
        image: &mut PxImageSliceMut<impl Pixel>,
        frame: impl Fn(UVec2) -> usize,
        filter: impl Fn(u8) -> u8,
    ) {
        self.1.draw((), image, frame, filter);
    }
}

impl Spatial for (PxRect, &PxFilterAsset) {
    fn frame_size(&self) -> UVec2 {
        *self.0
    }
}

pub(crate) type RectComponents<L> = (
    &'static PxRect,
    &'static PxFilter,
    &'static PxFilterLayers<L>,
    &'static PxPosition,
    &'static PxAnchor,
    &'static PxCanvas,
    Option<&'static PxAnimation>,
);

fn extract_rects<L: PxLayer>(
    rects: Extract<Query<(RectComponents<L>, &InheritedVisibility, RenderEntity)>>,
    mut cmd: Commands,
) {
    for ((&rect, filter, layers, &pos, &anchor, &canvas, animation), visibility, id) in &rects {
        let mut entity = cmd.entity(id);

        if !visibility.get() {
            entity.remove::<PxFilterLayers<L>>();
            continue;
        }

        entity.insert((rect, filter.clone(), layers.clone(), pos, anchor, canvas));

        if let Some(animation) = animation {
            entity.insert(*animation);
        } else {
            entity.remove::<PxAnimation>();
        }
    }
}

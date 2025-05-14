use bevy_derive::{Deref, DerefMut};
use bevy_math::{ivec2, uvec2};
use bevy_render::{sync_world::RenderEntity, Extract, RenderApp};

use crate::{
    animation::Animation, filter::DefaultPxFilterLayers, image::PxImageSliceMut, position::Spatial,
    prelude::*,
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

impl Default for PxRect {
    fn default() -> Self {
        Self(UVec2::ONE)
    }
}

impl Animation for (PxRect, &PxFilterAsset) {
    type Param = bool;

    fn frame_count(&self) -> usize {
        self.1.frame_count()
    }

    fn draw(
        &self,
        invert: bool,
        image: &mut PxImageSliceMut,
        frame: impl Fn(UVec2) -> usize,
        filter_fn: impl Fn(u8) -> u8,
    ) {
        let (_, PxFilterAsset(filter)) = self;

        for x in 0..image.image_width() as i32 {
            for y in 0..image.image_height() as i32 {
                let pos = ivec2(x, y);
                if image.contains_pixel(pos) != invert {
                    let pixel = image.image_pixel_mut(pos);
                    *pixel = filter_fn(filter.pixel(ivec2(
                        *pixel as i32,
                        frame(uvec2(x as u32, y as u32)) as i32,
                    )));
                }
            }
        }
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
    Has<PxInvertMask>,
);

fn extract_rects<L: PxLayer>(
    rects: Extract<Query<(RectComponents<L>, &InheritedVisibility, RenderEntity)>>,
    mut cmd: Commands,
) {
    for ((&rect, filter, layers, &pos, &anchor, &canvas, animation, invert), visibility, id) in
        &rects
    {
        let mut entity = cmd.entity(id);

        if !visibility.get() {
            entity.remove::<PxFilterLayers<L>>();
            continue;
        }

        entity.insert((rect, filter.clone(), layers.clone(), pos, anchor, canvas));

        if let Some(&animation) = animation {
            entity.insert(animation);
        } else {
            entity.remove::<PxAnimation>();
        }

        if invert {
            entity.insert(PxInvertMask);
        } else {
            entity.remove::<PxInvertMask>();
        }
    }
}

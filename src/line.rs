use bevy_derive::{Deref, DerefMut};
use bevy_math::{ivec2, uvec2};
use bevy_platform::collections::HashSet;
use bevy_render::{sync_world::RenderEntity, Extract, RenderApp};
use line_drawing::Bresenham;

use crate::{
    animation::{draw_frame, Frames},
    filter::DefaultPxFilterLayers,
    image::PxImageSliceMut,
    position::{PxLayer, Spatial},
    prelude::*,
};

pub(crate) fn plug<L: PxLayer>(app: &mut App) {
    app.sub_app_mut(RenderApp)
        .add_systems(ExtractSchedule, extract_lines::<L>);
}

/// Point list for a line
#[derive(Component, Deref, DerefMut, Clone, Default, Debug)]
#[require(DefaultPxFilterLayers, PxCanvas)]
pub struct PxLine(pub Vec<IVec2>);

impl Spatial for PxLine {
    fn frame_size(&self) -> UVec2 {
        if self.is_empty() {
            return UVec2::ZERO;
        }

        let (min, max) = self
            .iter()
            .copied()
            .fold((self[0], self[0]), |(min, max), point| {
                (min.min(point), max.max(point))
            });

        (max - min).as_uvec2()
    }
}

impl Frames for (&PxLine, &PxFilterAsset) {
    type Param = (IVec2, bool);

    fn frame_count(&self) -> usize {
        let (_, PxFilterAsset(filter)) = self;
        filter.area() / filter.width()
    }

    fn draw(
        &self,
        (offset, invert): Self::Param,
        image: &mut PxImageSliceMut,
        frame: impl Fn(UVec2) -> usize,
        _: impl Fn(u8) -> u8,
    ) {
        let (line, PxFilterAsset(filter)) = self;
        let mut poses = HashSet::new();

        for (start, end) in line.iter().zip(line.iter().skip(1)) {
            let start = *start + offset;
            let end = *end + offset;

            for pos in Bresenham::new(start.into(), end.into()) {
                poses.insert(IVec2::from(pos));
            }
        }

        let offset = image.offset();

        for x in 0..image.image_width() as i32 {
            for y in 0..image.image_height() as i32 {
                let pos = ivec2(x, y);

                if poses.contains(&(pos - offset)) != invert {
                    let pixel = image.image_pixel_mut(pos);
                    *pixel = filter.pixel(ivec2(
                        *pixel as i32,
                        frame(uvec2(x as u32, y as u32)) as i32,
                    ));
                }
            }
        }
    }
}

impl<T: IntoIterator<Item = IVec2>> From<T> for PxLine {
    fn from(line: T) -> Self {
        Self(line.into_iter().collect())
    }
}

pub(crate) type LineComponents<L> = (
    &'static PxLine,
    &'static PxFilter,
    &'static PxFilterLayers<L>,
    &'static PxCanvas,
    Option<&'static PxFrame>,
    Has<PxInvertMask>,
);

fn extract_lines<L: PxLayer>(
    lines: Extract<Query<(LineComponents<L>, &InheritedVisibility, RenderEntity)>>,
    mut cmd: Commands,
) {
    for ((line, filter, layers, &canvas, frame, invert), visibility, id) in &lines {
        let mut entity = cmd.entity(id);

        if !visibility.get() {
            entity.remove::<PxFilterLayers<L>>();
            continue;
        }

        entity.insert((line.clone(), filter.clone(), layers.clone(), canvas));

        if let Some(&frame) = frame {
            entity.insert(frame);
        } else {
            entity.remove::<PxFrame>();
        }

        if invert {
            entity.insert(PxInvertMask);
        } else {
            entity.remove::<PxInvertMask>();
        }
    }
}

pub(crate) fn draw_line(
    line: &PxLine,
    filter: &PxFilterAsset,
    invert: bool,
    image: &mut PxImageSliceMut,
    canvas: PxCanvas,
    frame: Option<PxFrame>,
    camera: PxCamera,
) {
    // TODO Make an `animated_line` example
    draw_frame(
        &(line, filter),
        (
            match canvas {
                PxCanvas::World => -*camera,
                PxCanvas::Camera => IVec2::ZERO,
            },
            invert,
        ),
        image,
        frame,
        [],
    );
}

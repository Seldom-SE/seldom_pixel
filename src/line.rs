use std::time::Duration;

use bevy::render::{sync_world::RenderEntity, Extract, RenderApp};
use line_drawing::Bresenham;

use crate::{
    animation::{draw_animation, Animation},
    filter::DefaultPxFilterLayers,
    image::PxImageSliceMut,
    pixel::Pixel,
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
        if self.len() == 0 {
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

impl Animation for (&PxLine, &PxFilterAsset) {
    type Param = IVec2;

    fn frame_count(&self) -> usize {
        let (_, PxFilterAsset(filter)) = self;
        filter.area() / filter.width()
    }

    fn draw(
        &self,
        param: Self::Param,
        image: &mut PxImageSliceMut<impl Pixel>,
        frame: impl Fn(UVec2) -> usize,
        _: impl Fn(u8) -> u8,
    ) {
        let (line, PxFilterAsset(filter)) = self;
        for (start, end) in line.iter().zip(line.iter().skip(1)) {
            let start = *start + param;
            let end = *end + param;

            for (x, y) in Bresenham::new(start.into(), end.into()) {
                if let Some(pixel) = image.get_pixel_mut((x, y).into()) {
                    if let Some(pixel) = pixel.get_value_mut() {
                        *pixel = filter.pixel(IVec2::new(
                            *pixel as i32,
                            frame(UVec2::new(x as u32, y as u32)) as i32,
                        ));
                    }
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
    Option<&'static PxAnimation>,
);

fn extract_lines<L: PxLayer>(
    lines: Extract<Query<(LineComponents<L>, &InheritedVisibility, RenderEntity)>>,
    mut cmd: Commands,
) {
    for ((line, filter, layers, &canvas, animation), visibility, id) in &lines {
        let mut entity = cmd.entity(id);

        if !visibility.get() {
            entity.remove::<PxFilterLayers<L>>();
            continue;
        }

        entity.insert((line.clone(), filter.clone(), layers.clone(), canvas));

        if let Some(animation) = animation {
            entity.insert(*animation);
        } else {
            entity.remove::<PxAnimation>();
        }
    }
}

pub(crate) fn draw_line(
    line: &PxLine,
    filter: &PxFilterAsset,
    image: &mut PxImageSliceMut<impl Pixel>,
    canvas: PxCanvas,
    animation: Option<(
        PxAnimationDirection,
        PxAnimationDuration,
        PxAnimationFinishBehavior,
        PxAnimationFrameTransition,
        Duration,
    )>,
    camera: PxCamera,
) {
    // TODO Make an `animated_line` example
    draw_animation(
        &(line, filter),
        match canvas {
            PxCanvas::World => -*camera,
            PxCanvas::Camera => IVec2::ZERO,
        },
        image,
        animation,
        [],
    );
}

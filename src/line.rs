use std::time::Duration;

use bevy::render::{Extract, RenderApp};
use line_drawing::Bresenham;

use crate::{
    animation::{draw_animation, Animation, AnimationComponents},
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

impl Animation for (&PxLine, &PxFilter) {
    type Param = IVec2;

    fn frame_count(&self) -> usize {
        let (_, PxFilter(filter)) = self;
        filter.area() / filter.width()
    }

    fn draw(
        &self,
        param: Self::Param,
        image: &mut PxImageSliceMut<impl Pixel>,
        frame: impl Fn(UVec2) -> usize,
        _: impl Fn(u8) -> u8,
    ) {
        let (line, PxFilter(filter)) = self;
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

/// Makes a line for a given list of points. Pixels are drawn by applying a filter.
#[derive(Bundle, Default)]
pub struct PxLineBundle<L: PxLayer> {
    /// A [`PxLine`] component
    pub line: PxLine,
    /// A [`PxFilterLayers`] component
    pub layers: PxFilterLayers<L>,
    /// A [`Handle<PxFilter>`] component
    pub filter: Handle<PxFilter>,
    /// A [`PxCanvas`] component
    pub canvas: PxCanvas,
    /// A [`Visibility`] component
    pub visibility: Visibility,
    /// An [`InheritedVisibility`] component
    pub inherited_visibility: InheritedVisibility,
}

pub(crate) type LineComponents<L> = (
    &'static PxLine,
    &'static Handle<PxFilter>,
    &'static PxFilterLayers<L>,
    &'static PxCanvas,
    Option<AnimationComponents>,
);

fn extract_lines<L: PxLayer>(
    lines: Extract<Query<(LineComponents<L>, &InheritedVisibility)>>,
    mut cmd: Commands,
) {
    for ((line, filter, layers, &canvas, animation), visibility) in &lines {
        if !visibility.get() {
            continue;
        }

        let mut line = cmd.spawn((line.clone(), filter.clone(), layers.clone(), canvas));

        if let Some((&direction, &duration, &on_finish, &frame_transition, &start)) = animation {
            line.insert((direction, duration, on_finish, frame_transition, start));
        }
    }
}

pub(crate) fn draw_line(
    line: &PxLine,
    filter: &PxFilter,
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

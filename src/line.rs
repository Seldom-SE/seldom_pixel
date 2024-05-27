use std::time::Duration;

use line_drawing::Bresenham;

use crate::{
    animation::{draw_animation, Animation},
    image::PxImageSliceMut,
    pixel::Pixel,
    position::{PxLayer, Spatial},
    prelude::*,
};

/// Point list for a line
#[derive(Component, Debug, Default, Deref, DerefMut)]
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

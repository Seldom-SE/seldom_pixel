//! Animation

use std::time::Duration;

use bevy_platform::time::Instant;

use crate::position::Spatial;
use crate::{image::PxImageSliceMut, prelude::*, set::PxSet};

pub(crate) fn plug(app: &mut App) {
    app.add_systems(
        PostUpdate,
        (
            update_animations::<PxSprite>,
            update_animations::<PxFilter>,
            update_animations::<PxText>,
            update_animations::<PxMap>,
        )
            .in_set(PxSet::FinishAnimations),
    );
}

#[derive(Clone, Copy)]
pub enum PxFrameSelector {
    Index(f32),
    Normalized(f32),
}

impl Default for PxFrameSelector {
    fn default() -> Self {
        Self::Normalized(0.)
    }
}

/// Method the animation uses to interpolate between frames
#[derive(Clone, Copy, Debug, Default)]
pub enum PxFrameTransition {
    /// Frames are not interpolated
    #[default]
    None,
    /// Dithering is used to interpolate between frames, smoothing the animation
    Dither,
}

#[derive(Component, Default, Clone, Copy)]
pub struct PxFrame {
    pub selector: PxFrameSelector,
    pub transition: PxFrameTransition,
}

impl From<PxFrameSelector> for PxFrame {
    fn from(value: PxFrameSelector) -> Self {
        Self {
            selector: value,
            ..default()
        }
    }
}

/// Direction the animation plays
#[derive(Clone, Copy, Debug, Default)]
pub enum PxAnimationDirection {
    /// The animation plays foreward
    #[default]
    Foreward,
    /// The animation plays backward
    Backward,
}

/// Animation duration
#[derive(Clone, Copy, Debug)]
pub enum PxAnimationDuration {
    /// Duration of the entire animation. When used on a tilemap, each tile's animation
    /// takes the same amount of time, but their frames may desync
    PerAnimation(Duration),
    /// Duration of each frame. When used on a tilemap, each frame will take the same amount
    /// of time, but the tile's animations may desync
    PerFrame(Duration),
}

impl Default for PxAnimationDuration {
    fn default() -> Self {
        Self::PerAnimation(Duration::from_secs(1))
    }
}

impl PxAnimationDuration {
    /// Creates a [`PxAnimationDuration::PerAnimation`] with the given number of milliseconds
    pub fn millis_per_animation(millis: u64) -> Self {
        Self::PerAnimation(Duration::from_millis(millis))
    }

    /// Creates a [`PxAnimationDuration::PerFrame`] with the given number of milliseconds
    pub fn millis_per_frame(millis: u64) -> Self {
        Self::PerFrame(Duration::from_millis(millis))
    }
}

/// Specifies what the animation does when it finishes
#[derive(Clone, Copy, Debug, Default)]
pub enum PxAnimationFinishBehavior {
    /// The entity is despawned when the animation finishes
    #[default]
    Despawn,
    /// [`PxAnimationFinished`] is added to the entity when the animation finishes
    Mark,
    /// A successful [`Done`] is added to the entity when the animation finishes
    #[cfg(feature = "state")]
    Done,
    /// The animation loops when it finishes
    Loop,
}

/// Animates an entity. Works on sprites, filters, text, tilemaps, rectangles, and lines.
#[derive(Component, Clone, Copy, Debug)]
#[require(PxFrame)]
pub struct PxAnimation {
    /// A [`PxAnimationDirection`]
    pub direction: PxAnimationDirection,
    /// A [`PxAnimationDuration`]
    pub duration: PxAnimationDuration,
    /// A [`PxAnimationFinishBehavior`]
    pub on_finish: PxAnimationFinishBehavior,
    /// Time when the animation started
    pub start: Instant,
}

impl Default for PxAnimation {
    fn default() -> Self {
        Self {
            direction: default(),
            duration: default(),
            on_finish: default(),
            start: Instant::now(),
        }
    }
}

/// Marks an animation that has finished. Automatically added to animations
/// with [`PxAnimationFinishBehavior::Mark`]
#[derive(Component, Debug)]
pub struct PxAnimationFinished;

pub(crate) trait Frames {
    type Param;

    fn frame_count(&self) -> usize;
    fn draw(
        &self,
        param: Self::Param,
        image: &mut PxImageSliceMut,
        frame: impl Fn(UVec2) -> usize,
        filter: impl Fn(u8) -> u8,
    );
}

pub(crate) trait AnimatedAssetComponent: Component {
    type Asset: Asset;

    fn handle(&self) -> &Handle<Self::Asset>;
    fn max_frame_count(asset: &Self::Asset) -> usize;
}

const DITHERING: [u16; 16] = [
    0b0000_0000_0000_0000,
    0b1000_0000_0000_0000,
    0b1000_0000_0010_0000,
    0b1010_0000_0010_0000,
    0b1010_0000_1010_0000,
    0b1010_0100_1010_0000,
    0b1010_0100_1010_0001,
    0b1010_0101_1010_0001,
    0b1010_0101_1010_0101,
    0b1110_0101_1010_0101,
    0b1110_0101_1011_0101,
    0b1111_0101_1011_0101,
    0b1111_0101_1111_0101,
    0b1111_1101_1111_0101,
    0b1111_1101_1111_0111,
    0b1111_1111_1111_0111,
];

pub(crate) fn animate(frame: PxFrame, frame_count: usize) -> impl Fn(UVec2) -> usize {
    let index = match frame.selector {
        PxFrameSelector::Normalized(frame) => frame * (frame_count - 1) as f32,
        PxFrameSelector::Index(frame) => frame,
    };

    let dithering = match frame.transition {
        PxFrameTransition::Dither => DITHERING[(index.fract() * 16.) as usize % 16],
        PxFrameTransition::None => 0,
    };
    let index = index.floor() as usize;

    move |pos| {
        (index + ((0b1000_0000_0000_0000 >> (pos.x % 4 + pos.y % 4 * 4)) & dithering != 0) as usize)
            % frame_count
    }
}

pub(crate) fn draw_frame<'a, A: Frames>(
    animation: &A,
    param: A::Param,
    image: &mut PxImageSliceMut,
    frame: Option<PxFrame>,
    filters: impl IntoIterator<Item = &'a PxFilterAsset>,
) {
    let frame_count = animation.frame_count();
    if frame_count == 0 {
        return;
    }

    let mut filter: Box<dyn Fn(u8) -> u8> = Box::new(|pixel| pixel);
    for filter_part in filters {
        let filter_part = filter_part.as_fn();
        filter = Box::new(move |pixel| filter_part(filter(pixel)));
    }

    if let Some(frame) = frame {
        let frame = animate(frame, frame_count);

        animation.draw(param, image, frame, filter);
    } else {
        let frame = |_| 0;
        animation.draw(param, image, frame, filter);
    }
}

pub(crate) fn draw_spatial<'a, A: Frames + Spatial>(
    spatial: &A,
    param: <A as Frames>::Param,
    image: &mut PxImageSliceMut,
    position: PxPosition,
    anchor: PxAnchor,
    canvas: PxCanvas,
    frame: Option<PxFrame>,
    filters: impl IntoIterator<Item = &'a PxFilterAsset>,
    camera: PxCamera,
) {
    let size = spatial.frame_size();
    let position = *position - anchor.pos(size).as_ivec2();
    let position = match canvas {
        PxCanvas::World => position - *camera,
        PxCanvas::Camera => position,
    };
    let position = IVec2::new(position.x, image.height() as i32 - position.y);
    let size = size.as_ivec2();

    let mut image = image.slice_mut(IRect {
        min: position - IVec2::new(0, size.y),
        max: position + IVec2::new(size.x, 0),
    });

    draw_frame(spatial, param, &mut image, frame, filters);
}

fn update_animations<A: AnimatedAssetComponent>(
    mut cmd: Commands,
    assets: Res<Assets<A::Asset>>,
    time: Res<Time<Real>>,
    mut animations: Query<(
        Entity,
        &mut PxFrame,
        &PxAnimation,
        Has<PxAnimationFinished>,
        &A,
    )>,
) {
    for (id, mut frame, animation, finished, a) in &mut animations {
        if let Some(asset) = assets.get(a.handle()) {
            let elapsed = time.last_update().unwrap_or_else(|| time.startup()) - animation.start;
            let max_frame_count = A::max_frame_count(asset);
            let lifetime = match animation.duration {
                PxAnimationDuration::PerAnimation(duration) => duration,
                PxAnimationDuration::PerFrame(duration) => duration * max_frame_count as u32,
            };

            let ratio = elapsed.div_duration_f32(lifetime);
            let ratio = match animation.on_finish {
                PxAnimationFinishBehavior::Despawn | PxAnimationFinishBehavior::Mark => {
                    ratio.min(1.)
                }
                #[cfg(feature = "state")]
                PxAnimationFinishBehavior::Done => ratio.min(1.),
                PxAnimationFinishBehavior::Loop => ratio.fract(),
            };
            let ratio = match animation.direction {
                PxAnimationDirection::Foreward => ratio,
                PxAnimationDirection::Backward => 1. + -ratio,
            };

            match frame.selector {
                PxFrameSelector::Index(ref mut index) => *index = max_frame_count as f32 * ratio,
                PxFrameSelector::Normalized(ref mut normalized) => *normalized = ratio,
            }

            if elapsed >= lifetime {
                match animation.on_finish {
                    PxAnimationFinishBehavior::Despawn => {
                        cmd.entity(id).despawn();
                    }
                    PxAnimationFinishBehavior::Mark => {
                        if !finished {
                            cmd.entity(id).insert(PxAnimationFinished);
                        }
                    }
                    #[cfg(feature = "state")]
                    PxAnimationFinishBehavior::Done => {
                        commands.entity(entity).insert(Done::Success);
                    }
                    PxAnimationFinishBehavior::Loop => (),
                }
            }
        }
    }
}

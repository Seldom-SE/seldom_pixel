//! Animation

use std::time::Duration;

use bevy::utils::Instant;

use crate::position::Spatial;
use crate::{
    image::{PxImage, PxImageSliceMut},
    pixel::Pixel,
    prelude::*,
    set::PxSet,
};

pub(crate) fn animation_plugin(app: &mut App) {
    app.configure_sets(
        PostUpdate,
        PxSet::FinishAnimations
            .after(PxSet::LoadAssets)
            .before(PxSet::Draw),
    )
    .add_systems(
        PostUpdate,
        (
            finish_animations::<PxSprite>,
            finish_animations::<PxFilter>,
            finish_animations::<PxTypeface>,
            finish_animations::<PxTileset>,
        )
            .in_set(PxSet::FinishAnimations),
    );
}

/// Direction the animation plays
#[derive(Clone, Component, Copy, Debug, Default)]
pub enum PxAnimationDirection {
    /// The animation plays foreward
    #[default]
    Foreward,
    /// The animation plays backward
    Backward,
}

/// Animation duration
#[derive(Clone, Component, Copy, Debug)]
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
#[derive(Clone, Component, Copy, Debug, Default)]
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

/// Method the animation uses to interpolate between frames
#[derive(Clone, Component, Copy, Debug, Default)]
pub enum PxAnimationFrameTransition {
    /// Frames are not interpolated
    #[default]
    None,
    /// Dithering is used to interpolate between frames, smoothing the animation
    Dither,
}

/// Time when the animation started
#[derive(Clone, Component, Copy, Debug, Deref, DerefMut)]
pub struct PxAnimationStart(pub Instant);

impl Default for PxAnimationStart {
    fn default() -> Self {
        Self(Instant::now())
    }
}

/// Animates an entity. Works on sprites, filters, text, tilemaps, and lines.
#[derive(Bundle, Clone, Copy, Debug, Default)]
pub struct PxAnimationBundle {
    /// A [`PxAnimationDirection`] component
    pub direction: PxAnimationDirection,
    /// A [`PxAnimationDuration`] component
    pub duration: PxAnimationDuration,
    /// A [`PxAnimationFinishBehavior`] component
    pub on_finish: PxAnimationFinishBehavior,
    /// A [`PxAnimationFrameTransition`] component
    pub frame_transition: PxAnimationFrameTransition,
    /// A [`PxAnimationStart`] component
    pub start: PxAnimationStart,
}

/// Marks an animation that has finished. Automatically added to animations
/// with [`PxAnimationFinishBehavior::Mark`]
#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct PxAnimationFinished;

pub(crate) trait Animation {
    type Param;

    fn frame_count(&self) -> usize;
    fn draw(
        &self,
        param: Self::Param,
        image: &mut PxImageSliceMut<impl Pixel>,
        frame: impl Fn(UVec2) -> usize,
        filter: impl Fn(u8) -> u8,
    );
}

pub(crate) trait AnimationAsset: Asset {
    fn max_frame_count(&self) -> usize;
}

static DITHERING: &[u16] = &[
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

pub(crate) fn animate(
    direction: PxAnimationDirection,
    duration: PxAnimationDuration,
    on_finish: PxAnimationFinishBehavior,
    frame_transition: PxAnimationFrameTransition,
    age: Duration,
    frame_count: usize,
) -> impl Fn(UVec2) -> usize {
    let (animation_duration, frame_duration) = match duration {
        PxAnimationDuration::PerAnimation(duration) => (duration, duration / frame_count as u32),
        PxAnimationDuration::PerFrame(duration) => (duration * frame_count as u32, duration),
    };
    let animation_millis = animation_duration.as_millis();
    let frame_millis = frame_duration.as_millis();

    let looping = match on_finish {
        PxAnimationFinishBehavior::Despawn | PxAnimationFinishBehavior::Mark => false,
        #[cfg(feature = "state")]
        PxAnimationFinishBehavior::Done => false,
        PxAnimationFinishBehavior::Loop => true,
    };

    let elapsed_millis = age.as_millis();
    let elapsed_millis = match looping {
        true => elapsed_millis % animation_millis,
        false => elapsed_millis,
    };
    let elapsed_millis = match direction {
        PxAnimationDirection::Foreward => match elapsed_millis > animation_millis {
            true => animation_millis,
            false => elapsed_millis,
        },
        PxAnimationDirection::Backward => match elapsed_millis > animation_millis {
            true => 0,
            false => animation_millis - elapsed_millis,
        },
    };

    let frame = ((elapsed_millis / frame_millis) as usize).min(frame_count - 1);

    let dithering = match frame_transition {
        PxAnimationFrameTransition::Dither if looping || frame + 1 < frame_count => {
            DITHERING[(elapsed_millis % frame_millis * 16 / frame_millis) as usize]
        }
        _ => 0,
    };

    move |pos| {
        (frame + (0b1000_0000_0000_0000 >> (pos.x % 4 + pos.y % 4 * 4) & dithering != 0) as usize)
            % frame_count
    }
}

pub(crate) fn draw_animation<'a, A: Animation>(
    animation: &A,
    param: A::Param,
    image: &mut PxImageSliceMut<impl Pixel>,
    animation_params: Option<(
        PxAnimationDirection,
        PxAnimationDuration,
        PxAnimationFinishBehavior,
        PxAnimationFrameTransition,
        Duration,
    )>,
    filters: impl IntoIterator<Item = &'a PxFilter>,
) {
    let mut filter: Box<dyn Fn(u8) -> u8> = Box::new(|pixel| pixel);
    for filter_part in filters {
        let filter_part = filter_part.as_fn();
        filter = Box::new(move |pixel| filter_part(filter(pixel)));
    }

    match animation_params {
        Some((direction, duration, on_finish, frame_transition, age)) => {
            let frame = animate(
                direction,
                duration,
                on_finish,
                frame_transition,
                age,
                animation.frame_count(),
            );

            animation.draw(param, image, frame, filter);
        }
        None => {
            let frame = |_| 0;

            animation.draw(param, image, frame, filter);
        }
    }
}

pub(crate) fn draw_spatial<'a, A: Animation + Spatial>(
    spatial: &A,
    param: <A as Animation>::Param,
    image: &mut PxImage<impl Pixel>,
    position: PxPosition,
    anchor: PxAnchor,
    canvas: PxCanvas,
    animation: Option<(
        PxAnimationDirection,
        PxAnimationDuration,
        PxAnimationFinishBehavior,
        PxAnimationFrameTransition,
        Duration,
    )>,
    filters: impl IntoIterator<Item = &'a PxFilter>,
    camera: PxCamera,
) {
    let size = spatial.frame_size();
    let position = *position - anchor.pos(size).as_ivec2();
    let position = match canvas {
        PxCanvas::World => position - *camera,
        PxCanvas::Camera => position,
    };

    let mut image = image.slice_mut(IRect {
        min: position,
        max: position + size.as_ivec2(),
    });

    draw_animation(spatial, param, &mut image, animation, filters);
}

pub(crate) fn copy_animation_params(
    params: Option<(
        &PxAnimationDirection,
        &PxAnimationDuration,
        &PxAnimationFinishBehavior,
        &PxAnimationFrameTransition,
        &PxAnimationStart,
    )>,
    time: &Time<Real>,
) -> Option<(
    PxAnimationDirection,
    PxAnimationDuration,
    PxAnimationFinishBehavior,
    PxAnimationFrameTransition,
    Duration,
)> {
    params.map(
        |(&direction, &duration, &on_finish, &frame_transition, &PxAnimationStart(start))| {
            (
                direction,
                duration,
                on_finish,
                frame_transition,
                time.last_update().unwrap_or_else(|| time.startup()) - start,
            )
        },
    )
}

fn finish_animations<A: AnimationAsset>(
    mut commands: Commands,
    animations: Query<(
        Entity,
        &Handle<A>,
        &PxAnimationDuration,
        &PxAnimationFinishBehavior,
        &PxAnimationStart,
        Option<&PxAnimationFinished>,
    )>,
    assets: Res<Assets<A>>,
    time: Res<Time<Real>>,
) {
    for (entity, animation, duration, on_finish, spawn_time, finished) in &animations {
        if let Some(animation) = assets.get(animation) {
            let lifetime = match duration {
                PxAnimationDuration::PerAnimation(duration) => *duration,
                PxAnimationDuration::PerFrame(duration) => {
                    *duration * animation.max_frame_count() as u32
                }
            };

            if time.last_update().unwrap_or_else(|| time.startup()) - **spawn_time >= lifetime {
                match on_finish {
                    PxAnimationFinishBehavior::Despawn => {
                        commands.entity(entity).despawn();
                    }
                    PxAnimationFinishBehavior::Mark => {
                        if finished.is_none() {
                            commands.entity(entity).insert(PxAnimationFinished);
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

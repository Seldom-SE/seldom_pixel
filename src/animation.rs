//! Animation

use std::time::Duration;

use bevy::utils::Instant;

#[cfg(feature = "map")]
use crate::map::PxTilesetData;
use crate::{
    asset::{PxAsset, PxAssetData},
    filter::PxFilterData,
    image::{PxImage, PxImageSliceMut},
    pixel::Pixel,
    prelude::*,
    set::PxSet,
    sprite::PxSpriteData,
    text::PxTypefaceData,
};

pub(crate) fn animation_plugin(app: &mut App) {
    app.configure_set(
        PxSet::FinishAnimations
            .after(PxSet::LoadAssets)
            .before(PxSet::Draw)
            .in_base_set(CoreSet::PostUpdate),
    )
    .add_systems(
        (
            remove_animation_time,
            insert_animation_time,
            apply_system_buffers,
        )
            .chain()
            .before(PxSet::FinishAnimations)
            .in_base_set(CoreSet::PostUpdate),
    )
    .add_systems(
        (
            finish_animations::<PxSpriteData>,
            finish_animations::<PxFilterData>,
            finish_animations::<PxTypefaceData>,
        )
            .in_set(PxSet::FinishAnimations),
    );

    #[cfg(feature = "map")]
    app.add_system(finish_animations::<PxTilesetData>.in_set(PxSet::FinishAnimations));
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
}

#[derive(Component, Debug, Deref, DerefMut)]
pub(crate) struct PxAnimationStart(Instant);

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

pub(crate) trait SpatialAnimation: Animation {
    fn frame_size(&self) -> UVec2;
}

pub(crate) trait AnimationAsset: PxAssetData {
    fn max_frame_count(&self) -> usize;
}

fn insert_animation_time(
    mut commands: Commands,
    animations: Query<Entity, Added<PxAnimationDuration>>,
    time: Res<Time>,
) {
    for animation in &animations {
        commands.entity(animation).insert(PxAnimationStart(
            time.last_update().unwrap_or_else(|| time.startup()),
        ));
    }
}

fn remove_animation_time(
    mut commands: Commands,
    mut animations: RemovedComponents<PxAnimationDuration>,
) {
    for animation in &mut animations {
        let Some(mut animation) = commands.get_entity(animation) else { continue };
        animation.remove::<PxAnimationStart>();
    }
}

static DITHERING: &[u16] = &[
    0b0000000000000000,
    0b1000000000000000,
    0b1000000000100000,
    0b1010000000100000,
    0b1010000010100000,
    0b1010010010100000,
    0b1010010010100001,
    0b1010010110100001,
    0b1010010110100101,
    0b1110010110100101,
    0b1110010110110101,
    0b1111010110110101,
    0b1111010111110101,
    0b1111110111110101,
    0b1111110111110111,
    0b1111111111110111,
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
    filters: impl IntoIterator<Item = &'a PxFilterData>,
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

pub(crate) fn draw_spatial<'a, A: SpatialAnimation>(
    spatial: &A,
    param: A::Param,
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
    filters: impl IntoIterator<Item = &'a PxFilterData>,
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
        Option<&PxAnimationStart>,
    )>,
    time: &Time,
) -> Option<(
    PxAnimationDirection,
    PxAnimationDuration,
    PxAnimationFinishBehavior,
    PxAnimationFrameTransition,
    Duration,
)> {
    params.map(
        |(direction, duration, on_finish, frame_transition, start)| {
            (
                *direction,
                *duration,
                *on_finish,
                *frame_transition,
                start
                    .map(|spawn_time| {
                        time.last_update().unwrap_or_else(|| time.startup()) - **spawn_time
                    })
                    .unwrap_or(Duration::ZERO),
            )
        },
    )
}

fn finish_animations<A: AnimationAsset>(
    mut commands: Commands,
    animations: Query<(
        Entity,
        &Handle<PxAsset<A>>,
        &PxAnimationDuration,
        &PxAnimationFinishBehavior,
        &PxAnimationStart,
        Option<&PxAnimationFinished>,
    )>,
    assets: Res<Assets<PxAsset<A>>>,
    time: Res<Time>,
) {
    for (entity, animation, duration, on_finish, spawn_time, finished) in &animations {
        if let Some(PxAsset::Loaded { asset: animation }) = assets.get(animation) {
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

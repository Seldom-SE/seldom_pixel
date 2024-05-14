//! Position, layers, velocity, anchors, etc.

use std::fmt::Debug;

use bevy::ecs::system::{SystemParam, SystemParamItem};

use crate::{
    map::PxTilesetData, math::Diagonal, prelude::*, screen::Screen, set::PxSet,
    sprite::PxSpriteData, system::SystemGet,
};

fn e<Out, Func: Send + Sync + 'static, F0: SystemParam>(_f: Func)
where
    for<'a> &'a mut Func: FnMut(F0) -> Out + FnMut(SystemParamItem<'_, '_, F0>) -> Out,
    Out: 'static,
{
}

pub(crate) fn position_plugin(app: &mut App) {
    e(align_to_screen::<(), &PxRect>);

    // app.add_systems(
    //     PostUpdate,
    //     (
    //         align_to_screen::<((), bool), (&PxMap, &PxTilesetData)>.before(PxSet::Draw),
    //         align_to_screen::<bool, &PxSpriteData>.before(PxSet::Draw),
    //         align_to_screen::<(), &PxRect>.before(PxSet::Draw),
    //         #[cfg(feature = "line")]
    //         align_to_screen::<(), &PxLine>.before(PxSet::Draw),
    //     )
    //         .before(PxSet::Draw),
    // );
}

pub(crate) trait Spatial {
    fn frame_size(&self) -> UVec2;
}

impl<T: Spatial> Spatial for &'_ T {
    fn frame_size(&self) -> UVec2 {
        (*self).frame_size()
    }
}

/// The position of an entity
#[derive(Clone, Component, Copy, Debug, Default, Deref, DerefMut)]
pub struct PxPosition(pub IVec2);

#[cfg(feature = "nav")]
impl Position2 for PxPosition {
    type Position = IVec2;

    fn get(&self) -> Self::Position {
        **self
    }

    fn set(&mut self, pos: Self::Position) {
        **self = pos;
    }
}

impl From<IVec2> for PxPosition {
    fn from(position: IVec2) -> Self {
        Self(position)
    }
}

/// Float-based position. Add to entities that have [`PxPosition`], but also need
/// a sub-pixel position. Use [`PxPosition`] unless a sub-pixel position is necessary.
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct PxSubPosition(pub Vec2);

impl From<Vec2> for PxSubPosition {
    fn from(vec: Vec2) -> Self {
        Self(vec)
    }
}

#[cfg(feature = "nav")]
impl Position2 for PxSubPosition {
    type Position = Vec2;

    fn get(&self) -> Self::Position {
        **self
    }

    fn set(&mut self, pos: Self::Position) {
        **self = pos;
    }
}

/// Velocity. Entities with this and [`PxSubPosition`] will move at this velocity over time.
#[derive(Clone, Component, Copy, Debug, Default, Deref, DerefMut)]
pub struct PxVelocity(pub Vec2);

impl From<Vec2> for PxVelocity {
    fn from(vec: Vec2) -> Self {
        Self(vec)
    }
}

fn update_sub_positions(mut query: Query<(&mut PxSubPosition, &PxVelocity)>, time: Res<Time>) {
    for (mut sub_position, velocity) in &mut query {
        if **velocity == Vec2::ZERO {
            let new_position = Vec2::new(sub_position.x.round(), sub_position.y.round());
            if **sub_position != new_position {
                **sub_position = new_position;
            }
        } else {
            **sub_position += **velocity * time.delta_seconds();
        }
    }
}

fn update_position_to_sub(
    mut query: Query<(&mut PxPosition, &PxSubPosition), Changed<PxSubPosition>>,
) {
    for (mut position, sub_position) in &mut query {
        let new_position = IVec2::new(sub_position.x.round() as i32, sub_position.y.round() as i32);
        if **position != new_position {
            **position = new_position;
        }
    }
}

/// Trait implemented for your game's custom layer type. Use the [`px_layer`] attribute
/// or derive/implement the required traits manually. The layers will be rendered in the order
/// defined by the [`PartialOrd`] implementation. So, lower values will be in the back
/// and vice versa.
pub trait PxLayer: Clone + Component + Debug + Default + Ord {}

impl<L: Clone + Component + Debug + Default + Ord> PxLayer for L {}

/// How a sprite is positioned relative to its [`PxPosition`]. It defaults to [`PxAnchor::Center`].
#[derive(Clone, Component, Copy, Debug, Default)]
pub enum PxAnchor {
    /// Center
    #[default]
    Center,
    /// Bottom left
    BottomLeft,
    /// Bottom center
    BottomCenter,
    /// Bottom right
    BottomRight,
    /// Center left
    CenterLeft,
    /// Center right
    CenterRight,
    /// Top left
    TopLeft,
    /// Top center
    TopCenter,
    /// Top right
    TopRight,
    /// Custom anchor. Values range from 0 to 1, from the bottom left to the top right.
    Custom(Vec2),
}

impl From<Vec2> for PxAnchor {
    fn from(vec: Vec2) -> Self {
        Self::Custom(vec)
    }
}

impl PxAnchor {
    pub(crate) fn x_pos(self, width: u32) -> u32 {
        match self {
            PxAnchor::BottomLeft | PxAnchor::CenterLeft | PxAnchor::TopLeft => 0,
            PxAnchor::BottomCenter | PxAnchor::Center | PxAnchor::TopCenter => width / 2,
            PxAnchor::BottomRight | PxAnchor::CenterRight | PxAnchor::TopRight => width,
            PxAnchor::Custom(anchor) => (width as f32 * anchor.x) as u32,
        }
    }

    pub(crate) fn y_pos(self, height: u32) -> u32 {
        match self {
            PxAnchor::BottomLeft | PxAnchor::BottomCenter | PxAnchor::BottomRight => 0,
            PxAnchor::CenterLeft | PxAnchor::Center | PxAnchor::CenterRight => height / 2,
            PxAnchor::TopLeft | PxAnchor::TopCenter | PxAnchor::TopRight => height,
            PxAnchor::Custom(anchor) => (height as f32 * anchor.y) as u32,
        }
    }

    pub(crate) fn pos(self, size: UVec2) -> UVec2 {
        UVec2::new(self.x_pos(size.x), self.y_pos(size.y))
    }
}

#[derive(Component)]
pub struct PxScreenAlign(pub Diagonal);

fn align_to_screen<'a, 'w: 'a, 's: 'a, M, T: Spatial + SystemGet<'a, M>>(
    // screen: Res<Screen>,
    // mut spatials: Query<(&PxScreenAlign, &PxAnchor, &mut PxPosition, Entity), T::Filter>,
    param: T::Param<'w, 's>,
) {
    // spatials
    //     .iter_mut()
    //     .for_each(|(PxScreenAlign(align), anchor, mut pos, spatial)| {
    //         let Some(t) = T::get(spatial, &param) else {
    //             return;
    //         };

    //         let size = t.frame_size();
    //         **pos = align.as_uvec2().as_ivec2() * (screen.size.as_ivec2() - size.as_ivec2())
    //             + anchor.pos(size).as_ivec2();
    //     });
}

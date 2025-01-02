//! Position, layers, velocity, anchors, etc.

use std::fmt::Debug;

use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    render::{extract_component::ExtractComponent, RenderApp},
};

use crate::{math::Diagonal, prelude::*, screen::Screen, set::PxSet, sprite::PxSpriteAsset};

// TODO Try to use traits instead of macros when we get the new solver
macro_rules! align_to_screen {
    ($components:ty, $param:ty, $get:expr) => {{
        fn align_to_screen(
            screen: Res<Screen>,
            mut spatials: Query<($components, &PxScreenAlign, &PxAnchor, &mut PxPosition)>,
            param: $param,
        ) {
            spatials
                .iter_mut()
                .for_each(|(components, PxScreenAlign(align), anchor, mut pos)| {
                    #[allow(clippy::redundant_closure_call)]
                    let Some(size) = ($get)(components, &param) else {
                        return;
                    };

                    **pos = align.as_uvec2().as_ivec2()
                        * (screen.computed_size.as_ivec2() - size.as_ivec2())
                        + anchor.pos(size).as_ivec2();
                });
        }

        align_to_screen
    }};
}

pub(crate) fn plug<L: PxLayer>(app: &mut App) {
    app.insert_resource(InsertDefaultLayer::new::<L>())
        .add_systems(
            PreUpdate,
            (
                update_sub_positions,
                update_position_to_sub.in_set(PxSet::UpdatePosToSubPos),
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            (
                align_to_screen!(&PxMap, Res<Assets<PxTileset>>, |map: &PxMap,
                                                                  tilesets: &Res<
                    Assets<PxTileset>,
                >| {
                    Some((&map.tiles, tilesets.get(&map.tileset)?).frame_size())
                }),
                align_to_screen!(
                    &PxSprite,
                    Res<Assets<PxSpriteAsset>>,
                    |sprite: &PxSprite, sprites: &Res<Assets<PxSpriteAsset>>| {
                        Some(sprites.get(&**sprite)?.frame_size())
                    }
                ),
                align_to_screen!(&PxRect, (), |rect: &PxRect, &()| Some(rect.frame_size())),
                #[cfg(feature = "line")]
                align_to_screen!(&PxLine, (), |line: &PxLine, &()| Some(line.frame_size())),
            ),
        )
        .sub_app_mut(RenderApp)
        .insert_resource(InsertDefaultLayer::new::<L>());
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
#[derive(ExtractComponent, Component, Deref, DerefMut, Clone, Copy, Default, Debug)]
pub struct PxPosition(pub IVec2);

impl From<IVec2> for PxPosition {
    fn from(position: IVec2) -> Self {
        Self(position)
    }
}

/// Trait implemented for your game's custom layer type. Use the [`px_layer`] attribute
/// or derive/implement the required traits manually. The layers will be rendered in the order
/// defined by the [`PartialOrd`] implementation. So, lower values will be in the back
/// and vice versa.
pub trait PxLayer: ExtractComponent + Component + Ord + Clone + Default + Debug {}

impl<L: ExtractComponent + Component + Ord + Clone + Default + Debug> PxLayer for L {}

#[derive(Resource, Deref)]
struct InsertDefaultLayer(Box<dyn Fn(&mut EntityWorldMut) + Send + Sync>);

impl InsertDefaultLayer {
    fn new<L: PxLayer>() -> Self {
        Self(Box::new(|entity| {
            entity.insert_if_new(L::default());
        }))
    }
}

#[derive(Component, Default)]
#[component(on_add = insert_default_layer)]
pub(crate) struct DefaultLayer;

fn insert_default_layer(mut world: DeferredWorld, entity: Entity, _: ComponentId) {
    world.commands().queue(move |world: &mut World| {
        let insert_default_layer = world.remove_resource::<InsertDefaultLayer>().unwrap();
        if let Ok(mut entity) = world.get_entity_mut(entity) {
            insert_default_layer(entity.remove::<DefaultLayer>());
        }
        world.insert_resource(insert_default_layer);
        // That's what it's all about!
    })
}

/// How a sprite is positioned relative to its [`PxPosition`]. It defaults to [`PxAnchor::Center`].
#[derive(ExtractComponent, Component, Clone, Copy, Default, Debug)]
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

/// Aligns a spatial entity to a corner of the screen
// TODO This is private because it's not done yet
#[derive(Component)]
struct PxScreenAlign(pub Diagonal);

/// Float-based position. Add to entities that have [`PxPosition`], but also need
/// a sub-pixel position. Use [`PxPosition`] unless a sub-pixel position is necessary.
#[derive(Component, Debug, Default, Deref, DerefMut)]
#[require(PxPosition)]
pub struct PxSubPosition(pub Vec2);

impl From<Vec2> for PxSubPosition {
    fn from(vec: Vec2) -> Self {
        Self(vec)
    }
}

#[cfg(feature = "nav")]
impl Position2 for PxSubPosition {
    fn get(&self) -> Vec2 {
        **self
    }

    fn set(&mut self, pos: Vec2) {
        **self = pos;
    }
}

/// Velocity. Entities with this and [`PxSubPosition`] will move at this velocity over time.
#[derive(Clone, Component, Copy, Debug, Default, Deref, DerefMut)]
#[require(PxSubPosition)]
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
            **sub_position += **velocity * time.delta_secs();
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

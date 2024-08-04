//! Bevy plugin for limited color palette pixel art games. Handles sprites, filters (defined
//! through images; apply to layers or individual entities), simple UI (text, buttons, and sprites
//! locked to the camera), tilemaps, animations (for sprites, filters, tilesets, and text;
//! supports dithering!), custom layers, particles (with pre-simulation!), palette changing,
//! typefaces, an in-game cursor, camera, lines, and more to come! Optional integration with
//! `seldom_state` (for animation state machines) and `seldom_map_nav`.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]
#![warn(missing_docs)]

pub mod animation;
mod button;
mod camera;
pub mod cursor;
pub mod filter;
mod image;
#[cfg(feature = "line")]
mod line;
mod map;
pub mod math;
pub mod palette;
#[cfg(feature = "particle")]
mod particle;
mod pixel;
pub mod position;
pub mod prelude;
pub mod screen;
pub mod set;
pub mod sprite;
mod text;
mod ui;

use std::{marker::PhantomData, path::PathBuf};

use position::PxLayer;
use prelude::*;

/// Add to your [`App`] to enable `seldom_pixel`. The type parameter is your custom layer type
/// used for z-ordering. You can make one using [`px_layer`].
#[derive(Debug)]
pub struct PxPlugin<L: PxLayer> {
    screen_size: ScreenSize,
    palette_path: PathBuf,
    _l: PhantomData<L>,
}

impl<L: PxLayer> PxPlugin<L> {
    /// Create a [`PxPlugin`]. `screen_size` is the size of the screen in pixels.
    /// `palette_path` is the path from `assets/` to your game's palette. This palette will be used
    /// to load assets, even if you change it later.
    pub fn new(screen_size: impl Into<ScreenSize>, palette_path: impl Into<PathBuf>) -> Self {
        Self {
            screen_size: screen_size.into(),
            palette_path: palette_path.into(),
            _l: PhantomData,
        }
    }
}

impl<L: PxLayer> Plugin for PxPlugin<L> {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            animation::plug,
            button::plug,
            camera::plug,
            cursor::plug,
            filter::plug::<L>,
            #[cfg(feature = "line")]
            line::plug::<L>,
            map::plug::<L>,
            palette::plug(self.palette_path.clone()),
            position::plug,
            screen::Plug::<L>::new(self.screen_size),
            sprite::plug::<L>,
            text::plug::<L>,
            #[cfg(feature = "particle")]
            (RngPlugin::default(), particle::plug::<L>),
        ));
    }
}

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

use animation::animation_plugin;
use button::button_plugin;
use camera::camera_plugin;
use cursor::cursor_plugin;
use filter::filter_plugin;
use map::map_plugin;
use palette::palette_plugin;
#[cfg(feature = "particle")]
use particle::particle_plugin;
use position::{position_plugin, PxLayer};
use prelude::*;
use screen::screen_plugin;
use seldom_fn_plugin::FnPluginExt;
use sprite::sprite_plugin;
use text::text_plugin;

/// Add to your [`App`] to enable `seldom_pixel`. The type parameter is your custom layer type
/// used for z-ordering. You can make one using [`px_layer`].
#[derive(Debug)]
pub struct PxPlugin<L: PxLayer> {
    screen_size: ScreenSize,
    palette_path: PathBuf,
    _l: PhantomData<L>,
}

impl<L: PxLayer> Plugin for PxPlugin<L> {
    fn build(&self, app: &mut App) {
        app.fn_plugin(px_plugin::<L>(self.screen_size, self.palette_path.clone()));
    }
}

impl<L: PxLayer> PxPlugin<L> {
    /// Create a [`PxPlugin`]. `screen_size` is the size of the screen in pixels.
    /// `palette_path` is the path from `assets/` to your game's palette. This palette will be used
    /// to load assets, even if you change it later.
    pub fn new(screen_size: impl Into<ScreenSize>, palette_path: impl Into<PathBuf>) -> Self {
        Self {
            screen_size: screen_size.into(),
            palette_path: palette_path.into(),
            _l: default(),
        }
    }
}

/// Function called by [`PxPlugin`]. You may instead call it directly or use `seldom_fn_plugin`,
/// which is another crate I maintain.
pub fn px_plugin<L: PxLayer>(
    screen_size: ScreenSize,
    palette_path: PathBuf,
) -> impl FnOnce(&mut App) {
    move |app| {
        app.fn_plugin(animation_plugin)
            .fn_plugin(button_plugin)
            .fn_plugin(camera_plugin)
            .fn_plugin(cursor_plugin)
            .fn_plugin(filter_plugin)
            .fn_plugin(map_plugin)
            .fn_plugin(palette_plugin(palette_path))
            .fn_plugin(position_plugin)
            .fn_plugin(screen_plugin::<L>(screen_size))
            .fn_plugin(sprite_plugin)
            .fn_plugin(text_plugin);
        #[cfg(feature = "particle")]
        app.add_plugins(RngPlugin::default())
            .fn_plugin(particle_plugin::<L>);
    }
}

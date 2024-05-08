# `seldom_pixel`

[![Crates.io](https://img.shields.io/crates/v/seldom_pixel.svg)](https://crates.io/crates/seldom_pixel)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/Seldom-SE/seldom_pixel#license)
[![Crates.io](https://img.shields.io/crates/d/seldom_pixel.svg)](https://crates.io/crates/seldom_pixel)

`seldom_pixel` is a Bevy plugin for limited color palette pixel art games. It handles:

- Sprites
- Filters (defined through images; apply to layers or individual entities)
- Simple UI (text, buttons, and sprites locked to the camera)
- Tilemaps
- Animations (for sprites, filters, tilesets, and text; supports dithering!)
- Custom layers
- Particles (with pre-simulation! Enable `particle` feature)
- Palette changing
- Typefaces
- An in-game cursor
- Camera
- Lines (enable `line` feature)
- And more to come!

It also features optional integration with:

- `seldom_state` (for animation state machines; `state` feature)
- `seldom_map_nav` (makes `SubPxPosition` implement `Position2`; `nav` feature)

See the `examples` directory for examples. If you need help, feel free to ping me
on [the Bevy Discord server](https://discord.com/invite/bevy) (`@Seldom`)! If any of the docs
need improvement, feel free to submit an issue or pr!

## Philosophies

- Assets are created through images

All assets, including filters, are loaded from images. `seldom_pixel`'s scope is limited
to rendered things, so this doesn't apply to things like levels and sounds. I recommend
finding an art program you're comfortable with. Personally, I use [GIMP](https://www.gimp.org/),
but it can be difficult to figure out. I hear good things
about [Aseprite](https://github.com/aseprite/aseprite/), which you can use for free if you
can compile it. I've only used this plugin on `.png` files, so I recommend using that format,
but feel free to try it on other lossless formats.

- It is what it looks like

This crate's position component, `PxPosition`, uses an `IVec2` (2-dimensional `i32` vector)
to store positions. This means that entities are located at exact pixel positions.
So, if it looks like the player is up against a wall, or a projectile hit an enemy, then the game
will respond like that's true. There is also a `SubPxPosition` component, which uses a `Vec2`,
for features like movement and velocity. It automatically updates the `PxPosition` component,
which I recommend using when possible. I also recommend resetting the `SubPxPosition`
to `PxPosition`'s value when it stops moving, so moving objects feel consistent to the player.
This is less of a concern for games with smaller pixels.

- Sacrifice versatility for productivity

If you are already interested in making a limited color palette pixel art game,
this is an easy win for you. Filters in `seldom_pixel` are just maps from each color
in the palette to another color in the palette. Filters like this would be difficult to create
for each of the 16,777,216 RGB colors, but `seldom_pixel` only allows up to 255 colors
in your palette (and you will likely want to use fewer), so it's easy to create effects.
This also applies on the library-development end too. The limitations of `seldom_pixel` mean
I only need to make its features work for 2D games that use bytes for pixels, so it's easier
to develop and maintain. Anyway, limitations can incite creativity.

## Future Work

This crate is currently in maintenance mode, so I'm not currently adding new features.

- [ ] Advanced UI, good enough to build a UI library on
- [ ] More advanced particle system
- [ ] More shape primitives
- [ ] Spatial filters that can filter defined areas, and apply their animations over space
      instead of time. For effects like lighting and bloom.
- [ ] Make the rendering happen in the render world

## Usage

Add to your `Cargo.toml`

```toml
# Replace * with your desired version
[dependencies]
seldom_pixel = "*"
```

Then add `PxPlugin` to your app. Check out the examples for further usage.

## Compatibility

| Bevy | `seldom_state` | `seldom_interop` | `bevy_ecs_tilemap` | `seldom_pixel` |
| ---- | -------------- | ---------------- | ------------------ | -------------- |
| 0.12 | 0.9            | 0.5              | 0.12               | 0.5            |
| 0.11 | 0.7            | 0.4              | 0.11               | 0.4            |
| 0.10 | 0.6            | 0.3              | 0.10               | 0.3            |
| 0.10 | 0.5            | 0.3              | 0.10               | 0.2            |
| 0.8  | 0.2            | 0.1              | 0.7                | 0.1            |

## License

`seldom_pixel` is dual-licensed under MIT and Apache 2.0 at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion
in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above,
without any additional terms or conditions.

## Demo Video

[![Demo video](https://img.youtube.com/vi/pmTPdGxYVYw/maxresdefault.jpg)](https://youtu.be/pmTPdGxYVYw)

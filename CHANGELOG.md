# Changelog

## 0.6.1 (2024-05-30)

### Fixed

- Fixed crash when drawing lines against the bounds of the screen

## 0.6 (2024-05-07)

### Changed

- Updated `bevy` to 0.13
- Replaced `bevy_ecs_tilemap` with built-in tilemap

## 0.5 (2024-02-16)

### Added

- `RectExt` extension trait for `IRect`, with some helper functions

### Changed

- Updated `bevy` to 0.12

### Removed

- `URect` and `IRect` in favor of `bevy`'s types of the same names

## 0.4 (2023-08-06)

### Changed

- Updated `bevy` to 0.11

## 0.3 (2023-05-07)

### Changed

- Updated `seldom_state` to 0.6

## 0.2.2 (2023-04-24)

### Fixed

- In wasm, spawning a particle emitter with pre-simulation too soon after startup caused a panic

## 0.2.1 (2023-04-15)

### Fixed

- Some setups had compile errors
- Wasm builds do not run (not fixed with the `particle` feature)

## 0.2 (2023-03-27)

### Changed

- Updated `bevy` to 0.10
- Text is drawn within a `PxRect` component
- Text requires `PxCanvas`

## 0.1.1 (2022-11-06)

### Fixed

- Animations spasmed when `PxAnimationBundle` is added, removed, and then added again

# Changelog

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

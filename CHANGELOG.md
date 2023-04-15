# Changelog

## 0.2.1 (2023-04-15)

### Fixed

- Some setups had compile errors, which are fixed
- Wasm builds run again, except with the `particle` feature

## 0.2 (2023-03-27)

### Changed

- Updated `bevy` to 0.10
- Text is drawn within a `PxRect` component
- Text requires `PxCanvas`

## 0.1.1 (2022-11-06)

### Fixed

- Animations don't spasm when `PxAnimationBundle` is added, removed, and then added again

# Changelog

## 0.8 (2025-01-01)

### Added

- `PxSprite` and `PxFilter` components to be used instead of `Handle<Px{Sprite,Filter}Asset>` (which
were called `Handle<Px{Sprite,Filter}>`)
- `PxAnimation` component
- `PxMap` component, which contains `PxTiles`, which was previously called `PxMap` and is no longer
a component
- `ScreenSize` enum, with a variant that allows dynamically changing draw resolution as the window's
aspect ratio changes. `PxPlugin` now accepts an `impl Into<ScreenSize>` for the screen size (which
may be a `UVec2`, like before).
- `PaletteHandle` resources, which contains the current `Handle<Palette>`
- `SelectLayerFn` trait, which layer selection functions must now implement. It has the additional
bound of `Clone`.
- `PaletteLoader` (`.px_palette.png`), `PxSpriteLoader` (`.px_sprite.png`), `PxFilterLoader`
(`.px_filter.png`), `PxTypefaceLoader` (`.px_typeface.png`) and `PxTilesetLoader`
(`.px_tileset.png`)
- `PxButtonSprite` and `PxButtonFilter` components
- `Orthogonal` and `Diagonal` math types

### Changed

- Updated `bevy` to 0.15
- `seldom_pixel` entities are extracted to the render world and drawn there. Involved components
implement `ExtractComponent` and involved resources implement `ExtractResource`. Due to this change,
entities on the same layer Z-fight. This behavior may change in the future.
- `PxSpriteData`, `PxFilter`, and `PxTilesetData` are now called `PxSpriteAsset`, `PxFilterAsset`,
and `PxTileset` respectively
- `PxAnimationDirection`, `PxAnimationDuration`, `PxAnimationFinishBehavior`, and
`PxAnimationFrameTransition` are no longer components and are instead fields of the new
`PxAnimation` component
- `PxText` has a `Handle<PxTypeface>` component, replacing the handle's use as a component
- `PxEmitterFrequency` and `PxEmitterSimulation` are no longer components and are instead fields of
the new `PxEmitter` component
- `Palette` is an asset instead of a resource
- `#[px_layer]` derives `ExtractComponent`
- `PxAnimationFinished`, `PxHover`, and `PxClick` are table components. They were sparse set.

### Removed

- The built-in asset management (`PxAsset`, `PxAssets`, `PxAssetTrait`, `PxAssetData`, and
`LoadingAssets`) in favor of the new asset loaders.
- Bundles (`PxSpriteBundle`, `PxFilterBundle`, `PxAnimationBundle`, `PxTextBundle`, `PxMapBundle`,
`PxTileBundle`, `PxEmitterBundle`, `PxButtonSpriteBundle`, and `PxButtonFilterBundle`) in favor of
required components
- `PxEmitterSprites`, `PxEmitterRange`, and `PxEmitterFn` in favor of `Vec<Handle<PxSprite>>`,
`IRect`, and `Box<dyn Fn(&mut EntityCommands) + Send + Sync>` fields in `PxEmitter`
- `PxIdleSprite`, `PxHoverSprite`, and `PxClickSprite` in favor of `PxButtonSprite`
- `PxIdleFilter`, `PxHoverFilter`, and `PxClickFilter` in favor of `PxButtonFilter`
- `PxAnimationStart` in favor of an `Instant` field in `PxAnimation`
- Vestigial variants of `PxSet` (`Unloaded`, `Loaded`, `LoadAssets`, `Draw`, and `DrawCursor`)

## 0.7 (2024-07-09)

### Changed

- Updated `bevy` to 0.14

### Removed

- `seldom_fn_plugin` integration

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

// In this program, anchors are demonstrated

use bevy::prelude::*;
use seldom_pixel::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: Vec2::splat(512.).into(),
                    ..default()
                }),
                ..default()
            }),
            PxPlugin::<Layer>::new(UVec2::splat(32), "palette/palette_1.palette.png".into()),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Centered
    commands.spawn(PxSpriteBundle::<Layer> {
        sprite: assets.load("sprite/mage.px_sprite.png"),
        position: IVec2::new(8, 16).into(),
        ..default()
    });

    // Bottom Left
    commands.spawn(PxSpriteBundle::<Layer> {
        sprite: assets.load("sprite/mage.px_sprite.png"),
        position: IVec2::splat(16).into(),
        anchor: PxAnchor::BottomLeft,
        ..default()
    });

    // Custom. Values range from 0 to 1, with the origin at the bottom left corner.
    commands.spawn(PxSpriteBundle::<Layer> {
        sprite: assets.load("sprite/mage.px_sprite.png"),
        position: IVec2::new(24, 16).into(),
        anchor: Vec2::new(0.2, 0.8).into(),
        ..default()
    });
}

#[px_layer]
struct Layer;

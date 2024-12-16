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
            PxPlugin::<Layer>::new(UVec2::splat(32), "palette/palette_1.palette.png"),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2d);

    // Centered
    commands.spawn((
        PxSprite(assets.load("sprite/mage.px_sprite.png")),
        PxPosition(IVec2::new(8, 16)),
    ));

    // Bottom Left
    commands.spawn((
        PxSprite(assets.load("sprite/mage.px_sprite.png")),
        PxPosition(IVec2::splat(16)),
        PxAnchor::BottomLeft,
    ));

    // Custom. Values range from 0 to 1, with the origin at the bottom left corner.
    commands.spawn((
        PxSprite(assets.load("sprite/mage.px_sprite.png")),
        PxPosition(IVec2::new(24, 16)),
        PxAnchor::Custom(Vec2::new(0.2, 0.8)),
    ));
}

#[px_layer]
struct Layer;

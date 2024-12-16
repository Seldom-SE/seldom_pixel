// In this program, a filter is applied to a single sprite

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

    // Spawn a sprite
    commands.spawn((
        PxSprite(assets.load("sprite/mage.px_sprite.png")),
        PxPosition(IVec2::new(8, 16)),
    ));

    // Spawn a sprite with a filter
    commands.spawn((
        PxSprite(assets.load("sprite/mage.px_sprite.png")),
        PxPosition(IVec2::new(24, 16)),
        PxFilter(assets.load("filter/invert.px_filter.png")),
    ));
}

#[px_layer]
struct Layer;

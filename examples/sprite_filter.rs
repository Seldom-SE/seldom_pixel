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
            PxPlugin::<Layer>::new(UVec2::splat(32), "palette/palette_1.palette.png".into()),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Spawn a sprite
    commands.spawn(PxSpriteBundle::<Layer> {
        sprite: assets.load("sprite/mage.px_sprite.png"),
        position: IVec2::new(8, 16).into(),
        ..default()
    });

    // Spawn a sprite with a filter
    commands.spawn((
        PxSpriteBundle::<Layer> {
            sprite: assets.load("sprite/mage.px_sprite.png"),
            position: IVec2::new(24, 16).into(),
            ..default()
        },
        assets.load::<PxFilter>("filter/invert.px_filter.png"),
    ));
}

#[px_layer]
struct Layer;

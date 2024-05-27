// In this program, a filter is used

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

    let mage = assets.load("sprite/mage.px_sprite.png");

    // Spawn some sprites
    commands.spawn(PxSpriteBundle::<Layer> {
        sprite: mage.clone(),
        position: IVec2::new(8, 16).into(),
        ..default()
    });

    commands.spawn(PxSpriteBundle::<Layer> {
        sprite: mage,
        position: IVec2::new(24, 16).into(),
        ..default()
    });

    // Spawn a filter
    commands.spawn(PxFilterBundle::<Layer> {
        filter: assets.load("filter/invert.px_filter.png"),
        ..default()
    });
}

#[px_layer]
struct Layer;

// In this program, a line is spawned

use bevy::prelude::*;
use seldom_pixel::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: Vec2::splat(512.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(PxPlugin::<Layer>::new(
            UVec2::splat(32),
            "palette/palette_1.png".into(),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(init)
        .run();
}

fn init(mut commands: Commands, mut sprites: PxAssets<PxSprite>, mut filters: PxAssets<PxFilter>) {
    commands.spawn(Camera2dBundle::default());

    let mage = sprites.load("sprite/mage.png");

    commands.spawn(PxSpriteBundle::<Layer> {
        sprite: mage,
        position: IVec2::splat(16).into(),
        ..default()
    });

    // Spawn a line. Layering and animation work the same as filters.
    commands.spawn(PxLineBundle::<Layer> {
        line: [(3, 22).into(), (31, 10).into()].into(),
        layers: PxFilterLayers::single_over(Layer),
        filter: filters.load("filter/invert.png"),
        ..default()
    });
}

#[px_layer]
struct Layer;

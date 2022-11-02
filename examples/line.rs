// In this program, a line is spawned

use bevy::prelude::*;
use seldom_pixel::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            width: 512.,
            height: 512.,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(PxPlugin::<Layer>::new(
            UVec2::splat(32),
            "palette/palette_1.png".into(),
        ))
        .add_startup_system(init)
        .run();
}

fn init(mut commands: Commands, mut sprites: PxAssets<PxSprite>, mut filters: PxAssets<PxFilter>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let mage = sprites.load("sprite/mage.png");

    commands.spawn_bundle(PxSpriteBundle::<Layer> {
        sprite: mage,
        position: IVec2::splat(16).into(),
        ..default()
    });

    // Spawn a line. Layering and animation work the same as filters.
    commands.spawn_bundle(PxLineBundle::<Layer> {
        line: [(3, 22).into(), (31, 10).into()].into(),
        layers: PxFilterLayers::single_over(Layer),
        filter: filters.load("filter/invert.png"),
        ..default()
    });
}

#[px_layer]
struct Layer;

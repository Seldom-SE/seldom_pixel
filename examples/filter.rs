// In this program, a filter is used

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

    // Spawn some sprites
    commands.spawn_bundle(PxSpriteBundle::<Layer> {
        sprite: mage.clone(),
        position: IVec2::new(8, 16).into(),
        ..default()
    });

    commands.spawn_bundle(PxSpriteBundle::<Layer> {
        sprite: mage,
        position: IVec2::new(24, 16).into(),
        ..default()
    });

    // Spawn a filter
    commands.spawn_bundle(PxFilterBundle::<Layer> {
        filter: filters.load("filter/invert.png"),
        ..default()
    });
}

#[px_layer]
struct Layer;

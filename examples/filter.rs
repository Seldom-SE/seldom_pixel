// In this program, a filter is used

use bevy::prelude::*;
use seldom_pixel::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: 512.,
                height: 512.,
                ..default()
            },
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
        filter: filters.load("filter/invert.png"),
        ..default()
    });
}

#[px_layer]
struct Layer;

// In this program, a sprite is spawned

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
            UVec2::splat(16),
            "palette/palette_1.png".into(),
        ))
        .add_startup_system(init)
        .run();
}

fn init(mut commands: Commands, mut sprites: PxAssets<PxSprite>) {
    commands.spawn_bundle(Camera2dBundle::default());

    // Spawn a sprite
    commands.spawn_bundle(PxSpriteBundle::<Layer> {
        sprite: sprites.load("sprite/mage.png"),
        position: IVec2::splat(8).into(),
        ..default()
    });
}

#[px_layer]
struct Layer;

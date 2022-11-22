// In this program, layers are demonstrated

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

fn init(mut commands: Commands, mut sprites: PxAssets<PxSprite>) {
    commands.spawn(Camera2dBundle::default());

    let mage = sprites.load("sprite/mage.png");

    // Spawn some sprites on different layers
    commands.spawn(PxSpriteBundle::<Layer> {
        sprite: mage.clone(),
        position: IVec2::new(10, 22).into(),
        ..default()
    });

    commands.spawn(PxSpriteBundle::<Layer> {
        sprite: mage.clone(),
        position: IVec2::new(14, 18).into(),
        layer: Layer::Middle(-1),
        ..default()
    });

    commands.spawn(PxSpriteBundle::<Layer> {
        sprite: mage.clone(),
        position: IVec2::new(18, 14).into(),
        layer: Layer::Middle(1),
        ..default()
    });

    commands.spawn(PxSpriteBundle::<Layer> {
        sprite: mage,
        position: IVec2::new(22, 10).into(),
        layer: Layer::Front,
        ..default()
    });
}

// Layers are in render order: back to front
#[px_layer]
enum Layer {
    #[default]
    Back,
    Middle(i32),
    Front,
}

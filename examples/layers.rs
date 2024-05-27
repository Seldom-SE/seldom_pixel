// In this program, layers are demonstrated

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

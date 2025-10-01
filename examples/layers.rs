// In this program, layers are demonstrated

use bevy::prelude::*;
use seldom_pixel::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: UVec2::splat(512).into(),
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

    let mage = assets.load("sprite/mage.px_sprite.png");

    // Spawn some sprites on different layers
    commands.spawn((PxSprite(mage.clone()), PxPosition(IVec2::new(10, 22))));

    commands.spawn((
        PxSprite(mage.clone()),
        PxPosition(IVec2::new(14, 18)),
        Layer::Middle(-1),
    ));

    commands.spawn((
        PxSprite(mage.clone()),
        PxPosition(IVec2::new(18, 14)),
        Layer::Middle(1),
    ));

    commands.spawn((PxSprite(mage), PxPosition(IVec2::new(22, 10)), Layer::Front));
}

// Layers are in render order: back to front
#[px_layer]
enum Layer {
    #[default]
    Back,
    Middle(i32),
    Front,
}

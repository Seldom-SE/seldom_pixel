// In this program, text is spawned

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
            PxPlugin::<Layer>::new(UVec2::splat(64), "palette/palette_1.png".into()),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn init(mut commands: Commands, mut typefaces: PxAssets<PxTypeface>) {
    commands.spawn(Camera2dBundle::default());

    // Spawn text
    commands.spawn(PxTextBundle::<Layer> {
        text: "THE MITOCHONDRIA IS THE POWERHOUSE OF THE CELL".into(),
        typeface: typefaces.load(
            "typeface/typeface.png",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            // Equivalent to, for example, `vec![PxSeparatorConfig { character: ' ', width: 4 }]`
            [(' ', 4)],
        ),
        rect: seldom_pixel::math::IRect::new(IVec2::ZERO, IVec2::splat(64)).into(),
        ..default()
    });
}

#[px_layer]
struct Layer;

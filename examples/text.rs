// In this program, text is spawned

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
            UVec2::new(64, 64),
            "palette/palette_1.png".into(),
        ))
        .add_startup_system(init)
        .run();
}

fn init(mut commands: Commands, mut typefaces: PxAssets<PxTypeface>) {
    commands.spawn_bundle(Camera2dBundle::default());

    // Spawn text
    commands.spawn_bundle(PxTextBundle::<Layer> {
        text: "THE MITOCHONDRIA IS THE POWERHOUSE OF THE CELL".into(),
        typeface: typefaces.load(
            "typeface/typeface.png",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            // Equivalent to, for example, `vec![PxSeparatorConfig { character: ' ', width: 4 }]`
            [(' ', 4)],
        ),
        ..default()
    });
}

#[px_layer]
struct Layer;

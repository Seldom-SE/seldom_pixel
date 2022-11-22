// In this program, text is spawned

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
            UVec2::new(64, 64),
            "palette/palette_1.png".into(),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(init)
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
        ..default()
    });
}

#[px_layer]
struct Layer;

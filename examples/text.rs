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
            PxPlugin::<Layer>::new(UVec2::splat(64), "palette/palette_1.palette.png"),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2d);

    // Spawn text
    commands.spawn((
        PxText {
            value: "THE MITOCHONDRIA IS THE POWERHOUSE OF THE CELL".to_string(),
            typeface: assets.load("typeface/typeface.px_typeface.png"),
        },
        PxRect(IRect::new(0, 0, 64, 64)),
    ));
}

#[px_layer]
struct Layer;

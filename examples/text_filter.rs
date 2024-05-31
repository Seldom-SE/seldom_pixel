// In this program, a filter is applied to text

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
    commands.spawn(Camera2dBundle::default());

    // Spawn text
    commands.spawn((
        PxTextBundle::<Layer> {
            text: "THE MITOCHONDRIA IS THE POWERHOUSE OF THE CELL".into(),
            typeface: assets.load("typeface/typeface.px_typeface.png"),
            rect: IRect::new(0, 0, 64, 64).into(),
            ..default()
        },
        assets.load::<PxFilter>("filter/dim.px_filter.png"),
    ));
}

#[px_layer]
struct Layer;

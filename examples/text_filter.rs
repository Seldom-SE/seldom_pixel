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

fn init(assets: Res<AssetServer>, mut cmd: Commands) {
    cmd.spawn(Camera2d);

    // Spawn text. Since we want the text wrap automatically, we wrap it in UI.
    PxContainer::build(
        PxText::build(
            "THE MITOCHONDRIA IS THE POWERHOUSE OF THE CELL",
            assets.load("typeface/typeface.px_typeface.png"),
        )
        .filter(assets.load("filter/dim.px_filter.png")),
    )
    .spawn(&mut cmd);
}

#[px_layer]
struct Layer;

// In this program, a tilemap is spawned

use bevy::prelude::*;
use rand::{Rng, thread_rng};
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
            PxPlugin::<Layer>::new(UVec2::splat(16), "palette/palette_1.palette.png"),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2d);

    let mut tiles = PxTiles::new(UVec2::splat(4));
    let mut rng = thread_rng();

    for x in 0..4 {
        for y in 0..4 {
            tiles.set(
                Some(commands.spawn(PxTile::from(rng.gen_range(0..4))).id()),
                UVec2::new(x, y),
            );
        }
    }

    // Spawn the map
    commands.spawn(PxMap {
        tiles,
        tileset: assets.load("tileset/tileset.px_tileset.png"),
    });
}

#[px_layer]
struct Layer;

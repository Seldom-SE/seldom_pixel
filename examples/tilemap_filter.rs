// In this program, a filter is applied to a tilemap and its tiles

use bevy::prelude::*;
use rand::{thread_rng, Rng};
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
            PxPlugin::<Layer>::new(UVec2::splat(16), "palette/palette_1.palette.png"),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2d);

    let mut tiles = PxTiles::new(UVec2::splat(4));
    let dim = assets.load("filter/dim.px_filter.png");
    let mut rng = thread_rng();

    for x in 0..4 {
        for y in 0..4 {
            // Each tile must be added to the `TileStorage`
            tiles.set(
                Some(
                    commands
                        .spawn((
                            PxTile::from(rng.gen_range(0..4)),
                            // Insert a filter on the tile
                            PxFilter(dim.clone()),
                        ))
                        .id(),
                ),
                UVec2::new(x, y),
            );
        }
    }

    // Spawn the map
    commands.spawn((
        PxMap {
            tiles,
            tileset: assets.load("tileset/tileset.px_tileset.png"),
        },
        // Insert a filter on the map. This filter applies to all tiles in the map.
        PxFilter(assets.load("filter/invert.px_filter.png")),
    ));
}

#[px_layer]
struct Layer;

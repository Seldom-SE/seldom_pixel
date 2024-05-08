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
            PxPlugin::<Layer>::new(UVec2::splat(16), "palette/palette_1.png".into()),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn init(
    mut commands: Commands,
    mut filters: PxAssets<PxFilter>,
    mut tilesets: PxAssets<PxTileset>,
) {
    commands.spawn(Camera2dBundle::default());

    let mut map = PxMap::new(UVec2::splat(4));
    let dim = filters.load("filter/dim.png");
    let mut rng = thread_rng();

    for x in 0..4 {
        for y in 0..4 {
            // Each tile must be added to the `TileStorage`
            map.set(
                Some(
                    commands
                        .spawn(PxTileBundle {
                            tile: rng.gen_range(0..4).into(),
                            ..default()
                        })
                        // Insert a filter on the tile
                        .insert(dim.clone())
                        .id(),
                ),
                UVec2::new(x, y),
            );
        }
    }

    // Spawn the map
    commands
        .spawn(PxMapBundle::<Layer> {
            map,
            tileset: tilesets.load("tileset/tileset.png", UVec2::splat(4)),
            ..default()
        })
        // Insert a filter on the map. This filter applies to all tiles in the map.
        .insert(filters.load("filter/invert.png"));
}

#[px_layer]
struct Layer;

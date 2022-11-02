// In this program, a filter is applied to a tilemap and its tiles

use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{thread_rng, Rng};
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
            UVec2::splat(16),
            "palette/palette_1.png".into(),
        ))
        .add_startup_system(init)
        .run();
}

fn init(
    mut commands: Commands,
    mut filters: PxAssets<PxFilter>,
    mut tilesets: PxAssets<PxTileset>,
) {
    commands.spawn_bundle(Camera2dBundle::default());

    let map_size = TilemapSize { x: 4, y: 4 };
    let dim = filters.load("filter/dim.png");
    let mut storage = TileStorage::empty(map_size);
    let mut rng = thread_rng();

    for x in 0..4 {
        for y in 0..4 {
            // Each tile must be added to the `TileStorage`
            storage.set(
                &TilePos { x, y },
                Some(
                    commands
                        .spawn_bundle(PxTileBundle {
                            texture: TileTexture(rng.gen_range(0..4)),
                            ..default()
                        })
                        // Insert a filter on the tile
                        .insert(dim.clone())
                        .id(),
                ),
            );
        }
    }

    // Spawn the map
    commands
        .spawn_bundle(PxMapBundle::<Layer> {
            size: map_size,
            storage,
            tileset: tilesets.load("tileset/tileset.png", UVec2::splat(4)),
            ..default()
        })
        // Insert a filter on the map. This filter applies to all tiles in the map.
        .insert(filters.load("filter/invert.png"));
}

#[px_layer]
struct Layer;

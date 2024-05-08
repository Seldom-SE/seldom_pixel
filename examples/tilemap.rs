// In this program, a tilemap is spawned

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

fn init(mut commands: Commands, mut tilesets: PxAssets<PxTileset>) {
    commands.spawn(Camera2dBundle::default());

    let mut map = PxMap::new(UVec2::splat(4));
    let mut rng = thread_rng();

    for x in 0..4 {
        for y in 0..4 {
            map.set(
                Some(
                    commands
                        .spawn(PxTileBundle {
                            tile: rng.gen_range(0..4).into(),
                            ..default()
                        })
                        .id(),
                ),
                UVec2::new(x, y),
            );
        }
    }

    // Spawn the map
    commands.spawn(PxMapBundle::<Layer> {
        map,
        tileset: tilesets.load("tileset/tileset.png", UVec2::splat(4)),
        ..default()
    });
}

#[px_layer]
struct Layer;

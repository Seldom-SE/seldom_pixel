// In this program, animated tilemaps are spawned

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

fn init(mut commands: Commands, mut tilesets: PxAssets<PxTileset>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let map_size = TilemapSize { x: 2, y: 4 };
    let mut storage = TileStorage::empty(map_size);
    let mut rng = thread_rng();

    for x in 0..2 {
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
                        .id(),
                ),
            );
        }
    }

    let tileset = tilesets.load("tileset/tileset.png", UVec2::splat(4));

    // Spawn the map
    commands
        .spawn_bundle(PxMapBundle::<Layer> {
            size: map_size,
            storage: storage.clone(),
            tileset: tileset.clone(),
            ..default()
        })
        .insert_bundle(PxAnimationBundle {
            // Use millis_per_animation to have each tile loop at the same time
            duration: PxAnimationDuration::millis_per_frame(250),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        });

    commands
        .spawn_bundle(PxMapBundle::<Layer> {
            size: map_size,
            storage,
            tileset,
            position: IVec2::new(8, 0).into(),
            ..default()
        })
        .insert_bundle(PxAnimationBundle {
            // Use millis_per_animation to have each tile loop at the same time
            duration: PxAnimationDuration::millis_per_frame(250),
            on_finish: PxAnimationFinishBehavior::Loop,
            frame_transition: PxAnimationFrameTransition::Dither,
            ..default()
        });
}

#[px_layer]
struct Layer;

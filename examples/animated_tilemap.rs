// In this program, animated tilemaps are spawned

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

    let mut tiles = PxTiles::new(UVec2::new(2, 4));
    let mut rng = thread_rng();

    for x in 0..2 {
        for y in 0..4 {
            tiles.set(
                Some(commands.spawn(PxTile::from(rng.gen_range(0..4))).id()),
                UVec2::new(x, y),
            );
        }
    }

    let tileset = assets.load("tileset/tileset.px_tileset.png");

    // Spawn the map
    commands.spawn((
        PxMap {
            tiles: tiles.clone(),
            tileset: tileset.clone(),
        },
        PxAnimation {
            // Use millis_per_animation to have each tile loop at the same time
            duration: PxAnimationDuration::millis_per_frame(250),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    commands.spawn((
        PxMap { tiles, tileset },
        PxPosition(IVec2::new(8, 0)),
        PxAnimation {
            // Use millis_per_animation to have each tile loop at the same time
            duration: PxAnimationDuration::millis_per_frame(250),
            on_finish: PxAnimationFinishBehavior::Loop,
            frame_transition: PxAnimationFrameTransition::Dither,
            ..default()
        },
    ));
}

#[px_layer]
struct Layer;

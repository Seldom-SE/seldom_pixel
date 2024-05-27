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
            PxPlugin::<Layer>::new(UVec2::splat(16), "palette/palette_1.palette.png".into()),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let mut map = PxMap::new(UVec2::new(2, 4));
    let mut rng = thread_rng();

    for x in 0..2 {
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

    let tileset = assets.load("tileset/tileset.px_tileset.png");

    // Spawn the map
    commands.spawn((
        PxMapBundle::<Layer> {
            map: map.clone(),
            tileset: tileset.clone(),
            ..default()
        },
        PxAnimationBundle {
            // Use millis_per_animation to have each tile loop at the same time
            duration: PxAnimationDuration::millis_per_frame(250),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    commands.spawn((
        PxMapBundle::<Layer> {
            map,
            tileset,
            position: IVec2::new(8, 0).into(),
            ..default()
        },
        PxAnimationBundle {
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

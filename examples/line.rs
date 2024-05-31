// In this program, a line is spawned

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
            PxPlugin::<Layer>::new(UVec2::splat(32), "palette/palette_1.palette.png"),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let mage = assets.load("sprite/mage.px_sprite.png");

    commands.spawn(PxSpriteBundle::<Layer> {
        sprite: mage,
        position: IVec2::splat(16).into(),
        ..default()
    });

    // Spawn a line. Layering and animation work the same as filters.
    commands.spawn(PxLineBundle::<Layer> {
        line: [(3, 22).into(), (31, 10).into()].into(),
        layers: PxFilterLayers::single_over(Layer),
        filter: assets.load("filter/invert.px_filter.png"),
        ..default()
    });
}

#[px_layer]
struct Layer;

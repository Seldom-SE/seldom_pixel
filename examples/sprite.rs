// In this program, a sprite is spawned

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
            PxPlugin::<Layer>::new(UVec2::splat(16), "palette/palette_1.palette.png".into()),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // Spawn a sprite
    commands.spawn(PxSpriteBundle::<Layer> {
        sprite: assets.load("sprite/mage.px_sprite.png"),
        position: IVec2::splat(8).into(),
        ..default()
    });
}

#[px_layer]
struct Layer;

// In this program, a filter is applied to a single sprite

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
            PxPlugin::<Layer>::new(UVec2::splat(32), "palette/palette_1.png".into()),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn init(mut commands: Commands, mut sprites: PxAssets<PxSprite>, mut filters: PxAssets<PxFilter>) {
    commands.spawn(Camera2dBundle::default());

    // Spawn a sprite
    commands.spawn(PxSpriteBundle::<Layer> {
        sprite: sprites.load("sprite/mage.png"),
        position: IVec2::new(8, 16).into(),
        ..default()
    });

    // Spawn a sprite with a filter
    commands.spawn((
        PxSpriteBundle::<Layer> {
            sprite: sprites.load("sprite/mage.png"),
            position: IVec2::new(24, 16).into(),
            ..default()
        },
        filters.load("filter/invert.png"),
    ));
}

#[px_layer]
struct Layer;

// In this program, a sprite can be flipped with X and Y keys.

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
            PxPlugin::<Layer>::new(UVec2::splat(16), "palette/palette_1.palette.png"),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .add_systems(Update, on_key)
        .run();
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2d);

    // Spawn a sprite
    commands.spawn((
        PxSprite(assets.load("sprite/mage.px_sprite.png")),
        PxPosition(IVec2::splat(8)),
        PxFlip::default(),
    ));
}

fn on_key(input: Res<ButtonInput<KeyCode>>, mut query: Query<&mut PxFlip>) {
    if input.just_pressed(KeyCode::KeyX) {
        for mut flip in &mut query {
            flip.x = !flip.x;
        }
    }

    if input.just_pressed(KeyCode::KeyY) {
        for mut flip in &mut query {
            flip.y = !flip.y;
        }
    }
}

#[px_layer]
struct Layer;

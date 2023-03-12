// In this program, an in-game cursor is used

use bevy::prelude::*;
use seldom_pixel::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: Vec2::splat(512.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(PxPlugin::<Layer>::new(
            UVec2::splat(16),
            "palette/palette_1.png".into(),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(init)
        .run();
}

fn init(
    mut commands: Commands,
    mut sprites: PxAssets<PxSprite>,
    mut filters: PxAssets<PxFilter>,
    mut cursor: ResMut<PxCursor>,
) {
    commands.spawn(Camera2dBundle::default());

    let idle = filters.load("filter/invert.png");

    // Switch to an in-game cursor. If the cursor feels like it lags behind,
    // consider `bevy_framepace` (https://github.com/aevyrie/bevy_framepace).
    *cursor = PxCursor::Filter {
        idle: idle.clone(),
        left_click: filters.load("filter/invert_dim.png"),
        right_click: idle,
    };

    // Sprite to show how the cursor's filter applies
    commands.spawn(PxSpriteBundle::<Layer> {
        sprite: sprites.load("sprite/mage.png"),
        position: IVec2::new(8, 8).into(),
        ..default()
    });
}

#[px_layer]
struct Layer;

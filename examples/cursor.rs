// In this program, an in-game cursor is used

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
        .run();
}

fn init(mut cursor: ResMut<PxCursor>, assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let idle = assets.load("filter/invert.px_filter.png");

    // Switch to an in-game cursor. If the cursor feels like it lags behind,
    // consider `bevy_framepace` (https://github.com/aevyrie/bevy_framepace).
    *cursor = PxCursor::Filter {
        idle: idle.clone(),
        left_click: assets.load("filter/invert_dim.px_filter.png"),
        right_click: idle,
    };

    // Sprite to show how the cursor's filter applies
    commands.spawn(PxSpriteBundle::<Layer> {
        sprite: assets.load("sprite/mage.px_sprite.png"),
        position: IVec2::new(8, 8).into(),
        ..default()
    });
}

#[px_layer]
struct Layer;

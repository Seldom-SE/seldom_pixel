// In this game, you can press buttons

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
        .add_systems(Update, interact_buttons)
        .run();
}

fn init(mut cursor: ResMut<PxCursor>, assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let idle = assets.load("filter/invert.px_filter.png");

    // Switch to an in-game cursor to show the player that they can click on things
    *cursor = PxCursor::Filter {
        idle: idle.clone(),
        left_click: assets.load("filter/invert_dim.px_filter.png"),
        right_click: idle,
    };

    let button_idle = assets.load("sprite/button_idle.px_sprite.png");

    // Sprite-based button
    commands.spawn((
        PxSpriteBundle::<Layer> {
            sprite: button_idle.clone(),
            position: IVec2::new(8, 4).into(),
            ..default()
        },
        PxButtonSpriteBundle {
            bounds: UVec2::new(8, 4).into(),
            idle: button_idle.clone().into(),
            hover: assets.load("sprite/button_hover.px_sprite.png").into(),
            click: assets.load("sprite/button_click.px_sprite.png").into(),
        },
        Button,
    ));

    // Filter-based button
    commands.spawn((
        PxSpriteBundle::<Layer> {
            sprite: button_idle,
            position: IVec2::new(8, 12).into(),
            ..default()
        },
        PxButtonFilterBundle {
            bounds: UVec2::new(8, 4).into(),
            idle: assets.load("filter/identity.px_filter.png").into(),
            hover: assets.load("filter/hover.px_filter.png").into(),
            click: assets.load("filter/click.px_filter.png").into(),
        },
        Button,
    ));
}

#[derive(Component)]
struct Button;

fn interact_buttons(
    hovers: Query<(), (With<Button>, Added<PxHover>)>,
    clicks: Query<(), (With<Button>, Added<PxClick>)>,
) {
    for _ in &hovers {
        info!("Hover!");
    }

    for _ in &clicks {
        info!("Click!");
    }
}

#[px_layer]
struct Layer;

// In this program, animated text is spawned

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
            PxPlugin::<Layer>::new(UVec2::splat(64), "palette/palette_1.palette.png".into()),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let typeface = assets.load("typeface/animated_typeface.px_typeface.png");

    // Spawn text
    commands.spawn((
        PxTextBundle::<Layer> {
            text: "LOOPED ANIMATION ⭐🙂⭐".into(),
            typeface: typeface.clone(),
            rect: IRect::new(0, 0, 64, 64).into(),
            alignment: PxAnchor::TopCenter,
            ..default()
        },
        PxAnimationBundle {
            // Use millis_per_animation to have each character loop at the same time
            duration: PxAnimationDuration::millis_per_frame(333),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    commands.spawn((
        PxTextBundle::<Layer> {
            text: "DITHERED ANIMATION 🙂⭐🙂".into(),
            typeface,
            rect: IRect::new(0, 0, 64, 64).into(),
            alignment: PxAnchor::BottomCenter,
            ..default()
        },
        PxAnimationBundle {
            // Use millis_per_animation to have each character loop at the same time
            duration: PxAnimationDuration::millis_per_frame(333),
            on_finish: PxAnimationFinishBehavior::Loop,
            frame_transition: PxAnimationFrameTransition::Dither,
            ..default()
        },
    ));
}

#[px_layer]
struct Layer;

// In this program, animated sprites are spawned

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
            PxPlugin::<Layer>::new(UVec2::new(51, 35), "palette/palette_1.palette.png"),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2d);

    // Load an animated sprite with `add_animated`
    let runner = assets.load("sprite/runner.px_sprite.png");

    // Despawn at the end
    commands.spawn((
        PxSprite(runner.clone()),
        PxAnchor::BottomLeft,
        PxAnimation::default(),
    ));

    // Add the `PxAnimationFinished` component at the end
    commands.spawn((
        PxSprite(runner.clone()),
        PxPosition(IVec2::new(13, 0)),
        PxAnchor::BottomLeft,
        PxAnimation {
            on_finish: PxAnimationFinishBehavior::Mark,
            ..default()
        },
    ));

    // Loop
    commands.spawn((
        PxSprite(runner.clone()),
        PxPosition(IVec2::new(26, 0)),
        PxAnchor::BottomLeft,
        PxAnimation {
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    // Backward
    commands.spawn((
        PxSprite(runner.clone()),
        PxPosition(IVec2::new(39, 0)),
        PxAnchor::BottomLeft,
        PxAnimation {
            direction: PxAnimationDirection::Backward,
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    // Faster
    commands.spawn((
        PxSprite(runner.clone()),
        PxPosition(IVec2::new(13, 18)),
        PxAnchor::BottomLeft,
        PxAnimation {
            duration: PxAnimationDuration::millis_per_animation(500),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    // Slower
    commands.spawn((
        PxSprite(runner.clone()),
        PxPosition(IVec2::new(0, 18)),
        PxAnchor::BottomLeft,
        PxAnimation {
            duration: PxAnimationDuration::millis_per_animation(2000),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    // Duration per frame
    commands.spawn((
        PxSprite(runner.clone()),
        PxPosition(IVec2::new(26, 18)),
        PxAnchor::BottomLeft,
        PxAnimation {
            duration: PxAnimationDuration::millis_per_frame(1000),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    // Dither between frames
    commands.spawn((
        PxSprite(runner),
        PxPosition(IVec2::new(39, 18)),
        PxAnchor::BottomLeft,
        PxAnimation {
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
        PxFrame {
            transition: PxFrameTransition::Dither,
            ..default()
        },
    ));
}

#[px_layer]
struct Layer;

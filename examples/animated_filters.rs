// In this program, animated filters are demonstrated

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

    let mage = assets.load("sprite/mage.px_sprite.png");

    // Spawn a bunch of sprites on different layers
    for layer in 0..8 {
        commands.spawn((
            PxSprite(mage.clone()),
            PxPosition(IVec2::new(layer % 4 * 13, layer / 4 * 18)),
            PxAnchor::BottomLeft,
            Layer(layer),
        ));
    }

    // Load the filter
    let fade_to_black = assets.load("filter/fade_to_black.px_filter.png");

    // Despawn at the end
    commands.spawn((
        PxFilter(fade_to_black.clone()),
        PxFilterLayers::single_clip(Layer(0)),
        PxAnimation::default(),
    ));

    // Add the `PxAnimationFinished` component at the end
    commands.spawn((
        PxFilter(fade_to_black.clone()),
        PxFilterLayers::single_clip(Layer(1)),
        PxAnimation {
            on_finish: PxAnimationFinishBehavior::Mark,
            ..default()
        },
    ));

    // Loop
    commands.spawn((
        PxFilter(fade_to_black.clone()),
        PxFilterLayers::single_clip(Layer(2)),
        PxAnimation {
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    // Backward
    commands.spawn((
        PxFilter(fade_to_black.clone()),
        PxFilterLayers::single_clip(Layer(3)),
        PxAnimation {
            direction: PxAnimationDirection::Backward,
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    // Faster
    commands.spawn((
        PxFilter(fade_to_black.clone()),
        PxFilterLayers::single_clip(Layer(5)),
        PxAnimation {
            duration: PxAnimationDuration::millis_per_animation(500),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    // Slower
    commands.spawn((
        PxFilter(fade_to_black.clone()),
        PxFilterLayers::single_clip(Layer(4)),
        PxAnimation {
            duration: PxAnimationDuration::millis_per_animation(2000),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    // Duration per frame
    commands.spawn((
        PxFilter(fade_to_black.clone()),
        PxFilterLayers::single_clip(Layer(6)),
        PxAnimation {
            duration: PxAnimationDuration::millis_per_frame(1000),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    // Dither between frames
    commands.spawn((
        PxFilter(fade_to_black),
        PxFilterLayers::single_clip(Layer(7)),
        PxAnimation {
            on_finish: PxAnimationFinishBehavior::Loop,
            frame_transition: PxAnimationFrameTransition::Dither,
            ..default()
        },
    ));
}

#[px_layer]
struct Layer(i32);

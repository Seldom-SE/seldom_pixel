// In this program, animated sprites are spawned

use bevy::prelude::*;
use seldom_pixel::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                width: 512.,
                height: 512.,
                ..default()
            },
            ..default()
        }))
        .add_plugin(PxPlugin::<Layer>::new(
            UVec2::new(51, 35),
            "palette/palette_1.png".into(),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(init)
        .run();
}

fn init(mut commands: Commands, mut sprites: PxAssets<PxSprite>) {
    commands.spawn(Camera2dBundle::default());

    // Load an animated sprite with `add_animated`
    let runner = sprites.load_animated("sprite/runner.png", 8);

    // Despawn at the end
    commands.spawn((
        PxSpriteBundle::<Layer> {
            sprite: runner.clone(),
            anchor: PxAnchor::BottomLeft,
            ..default()
        },
        PxAnimationBundle::default(),
    ));

    // Add the `PxAnimationFinished` component at the end
    commands.spawn((
        PxSpriteBundle::<Layer> {
            sprite: runner.clone(),
            position: IVec2::new(13, 0).into(),
            anchor: PxAnchor::BottomLeft,
            ..default()
        },
        PxAnimationBundle {
            on_finish: PxAnimationFinishBehavior::Mark,
            ..default()
        },
    ));

    // Loop
    commands.spawn((
        PxSpriteBundle::<Layer> {
            sprite: runner.clone(),
            position: IVec2::new(26, 0).into(),
            anchor: PxAnchor::BottomLeft,
            ..default()
        },
        PxAnimationBundle {
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    // Backward
    commands.spawn((
        PxSpriteBundle::<Layer> {
            sprite: runner.clone(),
            position: IVec2::new(39, 0).into(),
            anchor: PxAnchor::BottomLeft,
            ..default()
        },
        PxAnimationBundle {
            direction: PxAnimationDirection::Backward,
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    // Faster
    commands.spawn((
        PxSpriteBundle::<Layer> {
            sprite: runner.clone(),
            position: IVec2::new(13, 18).into(),
            anchor: PxAnchor::BottomLeft,
            ..default()
        },
        PxAnimationBundle {
            duration: PxAnimationDuration::millis_per_animation(500),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    // Slower
    commands.spawn((
        PxSpriteBundle::<Layer> {
            sprite: runner.clone(),
            position: IVec2::new(0, 18).into(),
            anchor: PxAnchor::BottomLeft,
            ..default()
        },
        PxAnimationBundle {
            duration: PxAnimationDuration::millis_per_animation(2000),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    // Duration per frame
    commands.spawn((
        PxSpriteBundle::<Layer> {
            sprite: runner.clone(),
            position: IVec2::new(26, 18).into(),
            anchor: PxAnchor::BottomLeft,
            ..default()
        },
        PxAnimationBundle {
            duration: PxAnimationDuration::millis_per_frame(1000),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    // Dither between frames
    commands.spawn((
        PxSpriteBundle::<Layer> {
            sprite: runner,
            position: IVec2::new(39, 18).into(),
            anchor: PxAnchor::BottomLeft,
            ..default()
        },
        PxAnimationBundle {
            on_finish: PxAnimationFinishBehavior::Loop,
            frame_transition: PxAnimationFrameTransition::Dither,
            ..default()
        },
    ));
}

#[px_layer]
struct Layer;

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
    commands.spawn(Camera2dBundle::default());

    let mage = assets.load("sprite/mage.px_sprite.png");

    // Spawn a bunch of sprites on different layers
    for layer in 0..8 {
        commands.spawn(PxSpriteBundle {
            sprite: mage.clone(),
            position: IVec2::new(layer % 4 * 13, layer / 4 * 18).into(),
            anchor: PxAnchor::BottomLeft,
            layer: Layer(layer),
            ..default()
        });
    }

    // Load the filter
    let fade_to_black = assets.load("filter/fade_to_black.px_filter.png");

    // Despawn at the end
    commands.spawn((
        PxFilterBundle {
            filter: fade_to_black.clone(),
            layers: PxFilterLayers::single_clip(Layer(0)),
            ..default()
        },
        PxAnimationBundle::default(),
    ));

    // Add the `PxAnimationFinished` component at the end
    commands.spawn((
        PxFilterBundle {
            filter: fade_to_black.clone(),
            layers: PxFilterLayers::single_clip(Layer(1)),
            ..default()
        },
        PxAnimationBundle {
            on_finish: PxAnimationFinishBehavior::Mark,
            ..default()
        },
    ));

    // Loop
    commands.spawn((
        PxFilterBundle {
            filter: fade_to_black.clone(),
            layers: PxFilterLayers::single_clip(Layer(2)),
            ..default()
        },
        PxAnimationBundle {
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
    ));

    // Backward
    commands.spawn((
        PxFilterBundle {
            filter: fade_to_black.clone(),
            layers: PxFilterLayers::single_clip(Layer(3)),
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
        PxFilterBundle {
            filter: fade_to_black.clone(),
            layers: PxFilterLayers::single_clip(Layer(5)),
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
        PxFilterBundle {
            filter: fade_to_black.clone(),
            layers: PxFilterLayers::single_clip(Layer(4)),
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
        PxFilterBundle {
            filter: fade_to_black.clone(),
            layers: PxFilterLayers::single_clip(Layer(6)),
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
        PxFilterBundle {
            filter: fade_to_black,
            layers: PxFilterLayers::single_clip(Layer(7)),
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
struct Layer(i32);

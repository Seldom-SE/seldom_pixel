// In this program, animated filters are demonstrated

use bevy::prelude::*;
use seldom_pixel::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor {
            width: 512.,
            height: 512.,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(PxPlugin::<Layer>::new(
            UVec2::new(51, 35),
            "palette/palette_1.png".into(),
        ))
        .add_startup_system(init)
        .run();
}

fn init(mut commands: Commands, mut sprites: PxAssets<PxSprite>, mut filters: PxAssets<PxFilter>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let mage = sprites.load("sprite/mage.png");

    // Spawn a bunch of sprites on different layers
    for layer in 0..8 {
        commands.spawn_bundle(PxSpriteBundle {
            sprite: mage.clone(),
            position: IVec2::new(layer % 4 * 13, layer / 4 * 18).into(),
            anchor: PxAnchor::BottomLeft,
            layer: Layer(layer),
            ..default()
        });
    }

    // Load the filter
    let fade_to_black = filters.load("filter/fade_to_black.png");

    // Despawn at the end
    commands
        .spawn_bundle(PxFilterBundle {
            filter: fade_to_black.clone(),
            layers: PxFilterLayers::single_clip(Layer(0)),
            ..default()
        })
        .insert_bundle(PxAnimationBundle::default());

    // Add the `PxAnimationFinished` component at the end
    commands
        .spawn_bundle(PxFilterBundle {
            filter: fade_to_black.clone(),
            layers: PxFilterLayers::single_clip(Layer(1)),
            ..default()
        })
        .insert_bundle(PxAnimationBundle {
            on_finish: PxAnimationFinishBehavior::Mark,
            ..default()
        });

    // Loop
    commands
        .spawn_bundle(PxFilterBundle {
            filter: fade_to_black.clone(),
            layers: PxFilterLayers::single_clip(Layer(2)),
            ..default()
        })
        .insert_bundle(PxAnimationBundle {
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        });

    // Backward
    commands
        .spawn_bundle(PxFilterBundle {
            filter: fade_to_black.clone(),
            layers: PxFilterLayers::single_clip(Layer(3)),
            ..default()
        })
        .insert_bundle(PxAnimationBundle {
            direction: PxAnimationDirection::Backward,
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        });

    // Faster
    commands
        .spawn_bundle(PxFilterBundle {
            filter: fade_to_black.clone(),
            layers: PxFilterLayers::single_clip(Layer(5)),
            ..default()
        })
        .insert_bundle(PxAnimationBundle {
            duration: PxAnimationDuration::millis_per_animation(500),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        });

    // Slower
    commands
        .spawn_bundle(PxFilterBundle {
            filter: fade_to_black.clone(),
            layers: PxFilterLayers::single_clip(Layer(4)),
            ..default()
        })
        .insert_bundle(PxAnimationBundle {
            duration: PxAnimationDuration::millis_per_animation(2000),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        });

    // Duration per frame
    commands
        .spawn_bundle(PxFilterBundle {
            filter: fade_to_black.clone(),
            layers: PxFilterLayers::single_clip(Layer(6)),
            ..default()
        })
        .insert_bundle(PxAnimationBundle {
            duration: PxAnimationDuration::millis_per_frame(1000),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        });

    // Dither between frames
    commands
        .spawn_bundle(PxFilterBundle {
            filter: fade_to_black,
            layers: PxFilterLayers::single_clip(Layer(7)),
            ..default()
        })
        .insert_bundle(PxAnimationBundle {
            on_finish: PxAnimationFinishBehavior::Loop,
            frame_transition: PxAnimationFrameTransition::Dither,
            ..default()
        });
}

#[px_layer]
struct Layer(i32);

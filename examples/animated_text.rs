// In this program, animated text is spawned

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
            UVec2::new(64, 64),
            "palette/palette_1.png".into(),
        ))
        .add_startup_system(init)
        .run();
}

fn init(mut commands: Commands, mut typefaces: PxAssets<PxTypeface>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let typeface = typefaces.load_animated(
        "typeface/animated_typeface.png",
        // See the function signature of `load_animated`
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ⭐🙂".chars().map(|character| {
            (
                character,
                match character == '⭐' {
                    true => 2,
                    false => 3,
                },
            )
        }),
        // Equivalent to, for example, `vec![PxSeparatorConfig { character: ' ', width: 4 }]`
        [(' ', 4)],
    );

    // Spawn text
    commands
        .spawn_bundle(PxTextBundle::<Layer> {
            text: "LOOPED ANIMATION ⭐🙂⭐".into(),
            typeface: typeface.clone(),
            alignment: PxAnchor::TopCenter,
            ..default()
        })
        .insert_bundle(PxAnimationBundle {
            // Use millis_per_animation to have each character loop at the same time
            duration: PxAnimationDuration::millis_per_frame(333),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        });

    commands
        .spawn_bundle(PxTextBundle::<Layer> {
            text: "DITHERED ANIMATION 🙂⭐🙂".into(),
            typeface,
            alignment: PxAnchor::BottomCenter,
            ..default()
        })
        .insert_bundle(PxAnimationBundle {
            // Use millis_per_animation to have each character loop at the same time
            duration: PxAnimationDuration::millis_per_frame(333),
            on_finish: PxAnimationFinishBehavior::Loop,
            frame_transition: PxAnimationFrameTransition::Dither,
            ..default()
        });
}

#[px_layer]
struct Layer;

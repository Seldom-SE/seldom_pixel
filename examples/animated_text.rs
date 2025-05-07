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
            PxPlugin::<Layer>::new(UVec2::splat(64), "palette/palette_1.palette.png"),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn init(assets: Res<AssetServer>, mut cmd: Commands) {
    cmd.spawn(Camera2d);

    let typeface = assets.load("typeface/animated_typeface.px_typeface.png");

    PxRow::build()
        .vertical()
        .entry(
            PxText::build("LOOPED ANIMATION ⭐🙂⭐", typeface.clone()).animation(PxAnimation {
                // Use millis_per_animation to have each character loop at the same time
                duration: PxAnimationDuration::millis_per_frame(333),
                on_finish: PxAnimationFinishBehavior::Loop,
                ..default()
            }),
        )
        .entry(PxRowSlot::build(PxSpace).stretch())
        .entry(
            PxText::build("DITHERED ANIMATION 🙂⭐🙂", typeface).animation(PxAnimation {
                // Use millis_per_animation to have each character loop at the same time
                duration: PxAnimationDuration::millis_per_frame(333),
                on_finish: PxAnimationFinishBehavior::Loop,
                frame_transition: PxAnimationFrameTransition::Dither,
                ..default()
            }),
        )
        .spawn_root(Layer, &mut cmd);
}

#[px_layer]
struct Layer;

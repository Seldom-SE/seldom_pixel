// In this program, animated text is spawned

use bevy::prelude::*;
use seldom_pixel::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: UVec2::splat(512).into(),
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

fn text(
    value: impl Into<String>,
    transition: PxFrameTransition,
    assets: &AssetServer,
) -> impl Bundle {
    (
        PxText::new(
            value,
            assets.load("typeface/animated_typeface.px_typeface.png"),
        ),
        PxAnimation {
            // Use millis_per_animation to have each character loop at the same time
            duration: PxAnimationDuration::millis_per_frame(333),
            on_finish: PxAnimationFinishBehavior::Loop,
            ..default()
        },
        PxFrame {
            transition,
            ..default()
        },
    )
}

fn init(assets: Res<AssetServer>, mut cmd: Commands) {
    cmd.spawn(Camera2d);

    cmd.spawn((
        Layer,
        PxUiRoot,
        PxRow {
            vertical: true,
            ..default()
        },
        children![
            text("LOOPED ANIMATION ⭐🙂⭐", PxFrameTransition::None, &assets),
            PxRowSlot { stretch: true },
            text(
                "DITHERED ANIMATION 🙂⭐🙂",
                PxFrameTransition::Dither,
                &assets
            ),
        ],
    ));
}

#[px_layer]
struct Layer;

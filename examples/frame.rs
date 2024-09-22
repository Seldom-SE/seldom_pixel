// In this program, two frames are spawned

use bevy::prelude::*;
use seldom_pixel::{frame::PxFrameBundle, prelude::*};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: Vec2::new(512., 384.).into(),
                    ..default()
                }),
                ..default()
            }),
            PxPlugin::<Layer>::new(UVec2::new(32, 24), "palette/palette_1.palette.png"),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(PxFrameBundle::<Layer> {
        frame: assets.load("frame/runner.px_frame.png"),
        offset: UVec2::new(2, 0).into(),
        size: UVec2::new(10, 15).into(),
        position: IVec2::new(1, 1).into(),
        anchor: PxAnchor::BottomLeft,
        ..default()
    });

    commands.spawn(PxFrameBundle::<Layer> {
        frame: assets.load("frame/runner.px_frame.png"),
        offset: UVec2::new(0, 18).into(),
        size: UVec2::new(11, 16).into(),
        position: IVec2::new(24, 20).into(),
        anchor: PxAnchor::TopCenter,
        ..default()
    });
}

#[px_layer]
struct Layer;

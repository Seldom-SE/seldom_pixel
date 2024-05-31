// In this game, you can move the camera with the arrow keys, and switch the mage's canvas
// by pressing space

use bevy::prelude::*;
use rand::{thread_rng, Rng};
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
        .add_systems(Update, (move_mage, move_camera, switch_canvas))
        .run();
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // `PxSubPosition` contains a `Vec2`. This is used
    // to represent the camera's sub-pixel position, which is rounded and applied
    // to the camera's pixel position.
    commands.spawn((PxSubPosition::default(), CameraPos));

    // By default, the mage is on the world canvas, which means you see it in different positions
    // based on where the camera is
    commands.spawn((
        PxSpriteBundle::<Layer> {
            sprite: assets.load("sprite/mage.px_sprite.png"),
            position: IVec2::splat(32).into(),
            ..default()
        },
        Mage,
    ));
}

#[derive(Component)]
struct CameraPos;

const CAMERA_SPEED: f32 = 10.;

// Move the camera based on the arrow keys
fn move_camera(
    mut camera_poses: Query<&mut PxSubPosition, With<CameraPos>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera: ResMut<PxCamera>,
) {
    let mut camera_pos = camera_poses.single_mut();
    **camera_pos += IVec2::new(
        keys.pressed(KeyCode::ArrowRight) as i32 - keys.pressed(KeyCode::ArrowLeft) as i32,
        keys.pressed(KeyCode::ArrowUp) as i32 - keys.pressed(KeyCode::ArrowDown) as i32,
    )
    .as_vec2()
    .normalize_or_zero()
        * time.delta_seconds()
        * CAMERA_SPEED;

    **camera = camera_pos.round().as_ivec2();
}

#[derive(Component)]
struct Mage;

// Jitter the mage around randomly. This function is framerate-sensitive, which is not good
// for a game, but it's fine for this example.
fn move_mage(mut mages: Query<&mut PxPosition, With<Mage>>) {
    if let Some(delta) =
        [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y].get(thread_rng().gen_range(0..50))
    {
        **mages.single_mut() += *delta;
    }
}

// Switch the canvas when you press space
fn switch_canvas(mut mages: Query<&mut PxCanvas>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Space) {
        let mut canvas = mages.single_mut();

        *canvas = match *canvas {
            // Camera means it is drawn relative to the camera, like UI
            PxCanvas::World => PxCanvas::Camera,
            // World means it is drawn relative to the world, like terrain
            PxCanvas::Camera => PxCanvas::World,
        };
    }
}

#[px_layer]
struct Layer;

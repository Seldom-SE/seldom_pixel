// In this game, you can press buttons

use std::fmt::Debug;

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
            PxPlugin::<Layer>::new(UVec2::splat(16), "palette/palette_1.palette.png"),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn set_sprite<E: Clone + Reflect + Debug>(
    path: &'static str,
) -> impl Fn(Trigger<Pointer<E>>, Query<&mut PxSprite>, Res<AssetServer>) {
    move |trigger, mut sprites, assets| {
        **sprites.get_mut(trigger.entity()).unwrap() = assets.load(path);
    }
}

fn init(mut cursor: ResMut<PxCursor>, assets: Res<AssetServer>, mut cmd: Commands) {
    cmd.spawn(Camera2d);

    let idle = assets.load("filter/invert.px_filter.png");

    // Switch to an in-game cursor to show the player that they can click on things
    *cursor = PxCursor::Filter {
        idle: idle.clone(),
        left_click: assets.load("filter/invert_dim.px_filter.png"),
        right_click: idle,
    };

    let idle_path = "sprite/button_idle.px_sprite.png";
    let hover_path = "sprite/button_hover.px_sprite.png";

    PxContainer::build(
        PxSprite::build(assets.load(idle_path))
            .observe(set_sprite::<Over>(hover_path))
            .observe(set_sprite::<Out>(idle_path))
            .observe(set_sprite::<Down>("sprite/button_click.px_sprite.png"))
            .observe(set_sprite::<Up>(hover_path))
            .observe(|_: Trigger<Pointer<Click>>| {
                info!("Click!");
            }),
    )
    .spawn(&mut cmd);
}

#[px_layer]
struct Layer;

// In this game, you can press space to cast a spell
// `seldom_state` is used to handle the animations

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use seldom_pixel::prelude::*;
use seldom_state::prelude::*;

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
            InputManagerPlugin::<Action>::default(),
            StateMachinePlugin,
            PxPlugin::<Layer>::new(UVec2::splat(16), "palette/palette_1.palette.png"),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

#[derive(Clone, Component)]
#[component(storage = "SparseSet")]
struct Idle;

#[derive(Clone, Component)]
#[component(storage = "SparseSet")]
struct Cast;

#[derive(Bundle)]
struct CastBundle {
    sprite: Handle<PxSprite>,
    animation: PxAnimationBundle,
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let idle = assets.load("sprite/mage.px_sprite.png");
    let cast = assets.load("sprite/mage_cast.px_sprite.png");

    // Spawn a sprite
    commands.spawn((
        PxSpriteBundle::<Layer> {
            sprite: idle.clone(),
            position: IVec2::splat(8).into(),
            ..default()
        },
        InputManagerBundle {
            input_map: InputMap::default()
                .insert(Action::Cast, KeyCode::Space)
                .build(),
            ..default()
        },
        StateMachine::default()
            .trans::<Idle, _>(just_pressed(Action::Cast), Cast)
            .on_enter::<Cast>(move |entity| {
                entity.insert(CastBundle {
                    sprite: cast.clone(),
                    animation: PxAnimationBundle {
                        duration: PxAnimationDuration::millis_per_animation(2000),
                        on_finish: PxAnimationFinishBehavior::Done,
                        ..default()
                    },
                });
            })
            .on_exit::<Cast>(|entity| {
                entity.remove::<CastBundle>();
            })
            .trans::<Cast, _>(done(None), Idle)
            .on_enter::<Idle>(move |entity| {
                entity.insert(idle.clone());
            })
            .set_trans_logging(true),
        Idle,
    ));
}

#[derive(Actionlike, Clone, Eq, Hash, PartialEq, Reflect)]
enum Action {
    Cast,
}

#[px_layer]
struct Layer;

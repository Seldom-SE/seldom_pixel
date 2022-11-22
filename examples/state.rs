// In this game, you can press space to cast a spell
// `seldom_state` is used to handle the animations

use bevy::prelude::*;
use seldom_pixel::prelude::*;
use seldom_state::prelude::*;

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
        .add_plugin(StateMachinePlugin)
        .add_plugin(TriggerPlugin::<CastPressed>::default())
        .add_plugin(PxPlugin::<Layer>::new(
            UVec2::splat(16),
            "palette/palette_1.png".into(),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(init)
        .run();
}

#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
struct Idle;

#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
struct Cast;

#[derive(FromReflect, Reflect)]
struct CastPressed;

impl Trigger for CastPressed {
    type Param<'w, 's> = Res<'w, Input<KeyCode>>;

    fn trigger(&self, _: Entity, keys: &Self::Param<'_, '_>) -> bool {
        keys.just_pressed(KeyCode::Space)
    }
}

#[derive(Bundle, Clone)]
struct CastBundle {
    sprite: Handle<PxSprite>,
    #[bundle]
    animation: PxAnimationBundle,
}

fn init(mut commands: Commands, mut sprites: PxAssets<PxSprite>) {
    commands.spawn(Camera2dBundle::default());

    let idle = sprites.load("sprite/mage.png");

    // Spawn a sprite
    commands.spawn((
        PxSpriteBundle::<Layer> {
            sprite: idle.clone(),
            position: IVec2::splat(8).into(),
            ..default()
        },
        StateMachine::new((Idle,))
            .trans::<(Idle,)>(CastPressed, (Cast,))
            .insert_on_enter::<(Cast,)>(CastBundle {
                sprite: sprites.load_animated("sprite/mage_cast.png", 4),
                animation: PxAnimationBundle {
                    duration: PxAnimationDuration::millis_per_animation(2000),
                    on_finish: PxAnimationFinishBehavior::Done,
                    ..default()
                },
            })
            .remove_on_exit::<(Cast,), PxAnimationBundle>()
            .trans::<(Cast,)>(DoneTrigger::Success, (Idle,))
            .insert_on_enter::<(Idle,)>((idle,)),
    ));
}

#[px_layer]
struct Layer;

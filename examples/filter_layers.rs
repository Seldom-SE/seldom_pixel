// In this game, you can change the filter's layer by pressing space

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
            UVec2::new(64, 32),
            "palette/palette_1.png".into(),
        ))
        .add_startup_system(init)
        .add_system(change_filter)
        .run();
}

struct GameAssets {
    invert: Handle<PxFilter>,
}

fn init(mut commands: Commands, mut sprites: PxAssets<PxSprite>, mut filters: PxAssets<PxFilter>) {
    commands.spawn_bundle(Camera2dBundle::default());

    commands.insert_resource(GameAssets {
        invert: filters.load("filter/invert.png"),
    });

    let mage = sprites.load("sprite/mage.png");

    // Spawn some sprites on different layers
    commands.spawn_bundle(PxSpriteBundle::<Layer> {
        sprite: mage.clone(),
        position: IVec2::new(8, 16).into(),
        ..default()
    });

    commands.spawn_bundle(PxSpriteBundle::<Layer> {
        sprite: mage.clone(),
        position: IVec2::new(24, 16).into(),
        layer: Layer::Middle(-1),
        ..default()
    });

    commands.spawn_bundle(PxSpriteBundle::<Layer> {
        sprite: mage.clone(),
        position: IVec2::new(40, 16).into(),
        layer: Layer::Middle(1),
        ..default()
    });

    commands.spawn_bundle(PxSpriteBundle::<Layer> {
        sprite: mage,
        position: IVec2::new(56, 16).into(),
        layer: Layer::Front,
        ..default()
    });
}

#[derive(Deref, DerefMut)]
struct CurrentFilter(i32);

impl Default for CurrentFilter {
    fn default() -> Self {
        Self(-1)
    }
}

fn change_filter(
    mut commands: Commands,
    mut current_filter: Local<CurrentFilter>,
    filters: Query<Entity, With<Handle<PxFilter>>>,
    assets: Res<GameAssets>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        **current_filter = (**current_filter + 1) % 4;

        for filter in &filters {
            commands.entity(filter).despawn();
        }

        commands.spawn_bundle(PxFilterBundle {
            filter: assets.invert.clone(),
            layers: match **current_filter {
                // Filters the Middle(-1) layer specifically
                0 => PxFilterLayers::single_clip(Layer::Middle(-1)),
                // Filters the screen's image after rendering Middle(0)
                1 => PxFilterLayers::single_over(Layer::Middle(0)),
                // Filters the Back and Front layers specifically
                2 => PxFilterLayers::Many(vec![Layer::Back, Layer::Front]),
                // Filters every layer matched by this `Fn`
                // Use `.into()` to convert a `Fn(&Layer) -> bool` to a `PxFilterLayers::Select`
                3 => (|layer: &Layer| matches!(layer, Layer::Middle(layer) if *layer >= 0)).into(),
                _ => unreachable!(),
            },
            ..default()
        });
    }
}

// Layers are in render order: back to front
#[px_layer]
enum Layer {
    #[default]
    Back,
    Middle(i32),
    Front,
}

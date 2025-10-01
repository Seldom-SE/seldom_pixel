// In this game, you can change the filter's layer by pressing space

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
            PxPlugin::<Layer>::new(UVec2::new(64, 32), "palette/palette_1.palette.png"),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .add_systems(Update, change_filter)
        .run();
}

#[derive(Resource)]
struct GameAssets {
    invert: Handle<PxFilterAsset>,
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.insert_resource(GameAssets {
        invert: assets.load("filter/invert.px_filter.png"),
    });

    let mage = assets.load("sprite/mage.px_sprite.png");

    // Spawn some sprites on different layers
    commands.spawn((PxSprite(mage.clone()), PxPosition(IVec2::new(8, 16))));

    commands.spawn((
        PxSprite(mage.clone()),
        PxPosition(IVec2::new(24, 16)),
        Layer::Middle(-1),
    ));

    commands.spawn((
        PxSprite(mage.clone()),
        PxPosition(IVec2::new(40, 16)),
        Layer::Middle(1),
    ));

    commands.spawn((PxSprite(mage), PxPosition(IVec2::new(56, 16)), Layer::Front));
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
    filters: Query<Entity, With<PxFilter>>,
    assets: Res<GameAssets>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        **current_filter = (**current_filter + 1) % 4;

        for filter in &filters {
            commands.entity(filter).despawn();
        }

        commands.spawn((
            PxFilter(assets.invert.clone()),
            match **current_filter {
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
        ));
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

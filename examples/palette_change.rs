// In this game, you can spawn a mage by pressing space and switch the palette by pressing tab

use bevy::prelude::*;
use rand::{thread_rng, Rng};
use seldom_pixel::{
    palette::{Palette, PaletteHandle},
    prelude::*,
};

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
            PxPlugin::<Layer>::new(
                UVec2::splat(64),
                // This is the palette that assets will be loaded with
                // It is also the palette that assets will be displayed with, until changed
                "palette/palette_1.palette.png".into(),
            ),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .add_systems(Update, (spawn_mage, change_palette))
        .run();
}

#[derive(Resource)]
struct GameAssets {
    // Palettes are created from normal images
    palette_1: Handle<Palette>,
    palette_2: Handle<Palette>,
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(GameAssets {
        palette_1: assets.load("palette/palette_1.palette.png"),
        palette_2: assets.load("palette/palette_2.palette.png"),
    });
}

fn spawn_mage(keys: Res<ButtonInput<KeyCode>>, assets: Res<AssetServer>, mut commands: Commands) {
    if keys.just_pressed(KeyCode::Space) {
        let mut rng = thread_rng();
        commands.spawn(PxSpriteBundle::<Layer> {
            // Usually, this sprite would be added in `init` to avoid duplicating data,
            // but it's here instead to show that loading assets is independent
            // of the current palette
            sprite: assets.load("sprite/mage.png"),
            position: IVec2::new(rng.gen_range(0..56), rng.gen_range(0..48)).into(),
            anchor: PxAnchor::BottomLeft,
            ..default()
        });
    }
}

// Marks which palette is currently loaded
#[derive(Default, Deref, DerefMut)]
struct CurrentPalette(bool);

fn change_palette(
    mut current_palette: Local<CurrentPalette>,
    mut palette: ResMut<PaletteHandle>,
    assets: Res<GameAssets>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        // Tab was pressed; switch palette
        **palette = if **current_palette {
            &assets.palette_1
        } else {
            &assets.palette_2
        }
        .clone();

        **current_palette = !**current_palette;
    }
}

#[px_layer]
struct Layer;

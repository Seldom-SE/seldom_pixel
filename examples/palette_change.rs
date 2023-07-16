// In this game, you can spawn a mage by pressing space and switch the palette by pressing tab

use bevy::prelude::*;
use rand::{thread_rng, Rng};
use seldom_pixel::{palette::Palette, prelude::*};

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
                "palette/palette_1.png".into(),
            ),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .add_systems(
            Update,
            (
                spawn_mage,
                change_palette.run_if(resource_exists::<Palette>()),
            ),
        )
        .run();
}

#[derive(Resource)]
struct GameAssets {
    // Palettes are created from normal images
    palette_1: Handle<Image>,
    palette_2: Handle<Image>,
}

fn init(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(GameAssets {
        palette_1: assets.load("palette/palette_1.png"),
        palette_2: assets.load("palette/palette_2.png"),
    });
}

fn spawn_mage(mut commands: Commands, mut sprites: PxAssets<PxSprite>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Space) {
        let mut rng = thread_rng();
        commands.spawn(PxSpriteBundle::<Layer> {
            // Usually, this sprite would be added in `init` to avoid duplicating data,
            // but it's here instead to show that loading assets is independent
            // of the current palette
            sprite: sprites.load("sprite/mage.png"),
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
    keys: Res<Input<KeyCode>>,
    assets: Res<GameAssets>,
    images: Res<Assets<Image>>,
    mut palette: ResMut<Palette>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        // Tab was pressed; switch palette
        // `if let Some` to make sure the image is loaded already
        if let Some(palette_image) = images.get(match **current_palette {
            true => &assets.palette_1,
            false => &assets.palette_2,
        }) {
            **current_palette = !**current_palette;
            // Create a new palette
            *palette = Palette::new(palette_image);
        }
    }
}

#[px_layer]
struct Layer;

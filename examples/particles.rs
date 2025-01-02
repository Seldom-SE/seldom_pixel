// In this program, a particle emitter is spawned

use std::time::Duration;

use bevy::{ecs::system::EntityCommands, prelude::*};
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
            PxPlugin::<Layer>::new(UVec2::splat(32), "palette/palette_1.palette.png"),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, init)
        .run();
}

fn init(assets: Res<AssetServer>, mut commands: Commands) {
    commands.spawn(Camera2d);

    // Spawn a particle emitter
    commands
        .spawn((
            PxEmitter {
                sprites: vec![
                    assets.load("sprite/snow_1.px_sprite.png"),
                    assets.load("sprite/snow_2.px_sprite.png"),
                ],
                // Range where the particles can spawn
                range: IRect::new(-4, 36, 36, 36),
                // Range of how often the particles spawn
                frequency: PxEmitterFrequency::new(
                    Duration::from_millis(800),
                    Duration::from_millis(1500),
                ),
                // `PxEmitterSimulation::Simulate` spawns particles
                // as soon as the `PxEmitterBundle` is spawned, with values as if they had been spawned
                // earlier. This is useful when an emitter comes into view,
                // and you want it to look like it had been emitting particles all along.
                simulation: PxEmitterSimulation::Simulate,
                // This function is run on each particle that spawns. It is run
                // after all of the other components are added, so you can use this to override components.
                on_spawn: Box::new(|particle: &mut EntityCommands| {
                    // Let's make each particle animated
                    particle.insert(PxAnimation {
                        on_finish: PxAnimationFinishBehavior::Loop,
                        ..default()
                    });
                }),
            },
            // Particle lifetime
            PxParticleLifetime(Duration::from_secs(30)),
            // Particle starting velocity
            PxVelocity(Vec2::new(0., -2.5)),
        ))
        .log_components();
}

#[px_layer]
struct Layer;

//! Particles and particle emitters

use std::time::Duration;

use bevy::{ecs::system::EntityCommands, utils::Instant};

use crate::{math::IRect, position::PxLayer, prelude::*, set::PxSet};

// https://github.com/bevyengine/bevy/issues/8483
// In wasm, time starts at 0, so it needs an offset to represent an instant before the app started.
// If a day isn't sufficient for your use case, file an issue!
const TIME_OFFSET: Duration = Duration::from_secs(60 * 60 * 24);

pub(crate) fn particle_plugin<L: PxLayer>(app: &mut App) {
    app.configure_sets(PostUpdate, PxSet::UpdateEmitters.before(PxSet::Draw))
        .add_systems(
            PostUpdate,
            (
                (
                    (simulate_emitters::<L>, insert_emitter_time),
                    (apply_deferred, update_emitters::<L>)
                        .chain()
                        .in_set(PxSet::UpdateEmitters),
                )
                    .chain(),
                despawn_particles.before(PxSet::Draw),
            ),
        );
}

/// Possible sprites for an emitter's particles
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct PxEmitterSprites(pub Vec<Handle<PxSprite>>);

impl<T: IntoIterator<Item = Handle<PxSprite>>> From<T> for PxEmitterSprites {
    fn from(t: T) -> Self {
        Self(t.into_iter().collect())
    }
}

/// Location range for an emitter's particles
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct PxEmitterRange(pub IRect);

impl From<IRect> for PxEmitterRange {
    fn from(rect: IRect) -> Self {
        Self(rect)
    }
}

/// A particle's lifetime
#[derive(Clone, Component, Copy, Debug, Deref, DerefMut)]
pub struct PxParticleLifetime(pub Duration);

impl Default for PxParticleLifetime {
    fn default() -> Self {
        Self(Duration::from_secs(1))
    }
}

impl From<Duration> for PxParticleLifetime {
    fn from(duration: Duration) -> Self {
        Self(duration)
    }
}

/// Spawn frequency range for an emitter
#[derive(Component, Debug)]
pub struct PxEmitterFrequency {
    min: Duration,
    max: Duration,
    next: Option<Duration>,
}

impl Default for PxEmitterFrequency {
    fn default() -> Self {
        Self::single(Duration::from_secs(1))
    }
}

impl PxEmitterFrequency {
    /// Create a new [`PxEmitterFrequency`] with frequency bounds
    pub fn new(min: Duration, max: Duration) -> Self {
        Self {
            min,
            max,
            next: None,
        }
    }

    /// Create a [`PxEmitterFrequency`] with a certain frequency
    pub fn single(duration: Duration) -> Self {
        Self {
            min: duration,
            max: duration,
            next: None,
        }
    }

    fn next(&mut self, rng: &mut Rng) -> Duration {
        if let Some(duration) = self.next {
            duration
        } else {
            let duration = (self.max - self.min).mul_f32(rng.f32()) + self.min;
            self.next = Some(duration);
            duration
        }
    }

    fn update_next(&mut self, rng: &mut Rng) -> Duration {
        let duration = self.next(rng);
        self.next = None;
        self.next(rng);
        duration
    }
}

/// Determines whether the emitter is pre-simulated
#[derive(Component, Debug, Default, Eq, PartialEq)]
pub enum PxEmitterSimulation {
    /// The emitter is not pre-simulated
    #[default]
    None,
    /// The emitter is pre-simulated. This means that the emitter will spawn particles
    /// as soon as the `PxEmitterBundle` is spawned, with values as if they had been spawned
    /// earlier. This is useful when an emitter comes into view,
    /// and you want it to look like it had been emitting particles all along.
    Simulate,
}

/// This function is run on each particle that spawns. It is run
/// after all of the other components are added, so you can use this to override components.
#[derive(Component, Deref, DerefMut)]
pub struct PxEmitterFn(pub Box<dyn Fn(&mut EntityCommands) + Send + Sync>);

impl Default for PxEmitterFn {
    fn default() -> Self {
        (|_: &mut EntityCommands| ()).into()
    }
}

impl<T: 'static + Fn(&mut EntityCommands) + Send + Sync> From<T> for PxEmitterFn {
    fn from(t: T) -> Self {
        Self(Box::new(t))
    }
}

/// Creates a particle emitter
#[derive(Bundle, Default)]
pub struct PxEmitterBundle<L: PxLayer> {
    /// A [`PxEmitterSprites`] component
    pub sprites: PxEmitterSprites,
    /// A [`PxEmitterRange`] component
    pub range: PxEmitterRange,
    /// A [`PxAnchor`] component; added to the particles
    pub anchor: PxAnchor,
    /// A layer component; added to the particles
    pub layer: L,
    /// A [`PxCanvas`] component; added to the particles
    pub canvas: PxCanvas,
    /// A [`PxParticleLifetime`] component; added to the particles
    pub lifetime: PxParticleLifetime,
    /// A [`PxVelocity`] component; added to the particles
    pub velocity: PxVelocity,
    /// A [`PxEmitterFrequency`] component
    pub frequency: PxEmitterFrequency,
    /// A [`PxEmitterSimulation`] component
    pub simulation: PxEmitterSimulation,
    /// A [`PxEmitterFn`] component
    pub on_spawn: PxEmitterFn,
}

#[derive(Component, Debug, Deref, DerefMut)]
struct PxEmitterStart(Instant);

#[derive(Component, Debug, Deref, DerefMut)]
struct PxParticleStart(Instant);

impl Default for PxParticleStart {
    fn default() -> Self {
        Self(Instant::now())
    }
}

impl From<Instant> for PxParticleStart {
    fn from(duration: Instant) -> Self {
        Self(duration)
    }
}

#[derive(Bundle, Default)]
struct PxParticleBundle {
    position: PxSubPosition,
    velocity: PxVelocity,
    start: PxParticleStart,
    lifetime: PxParticleLifetime,
}

fn simulate_emitters<L: PxLayer>(
    mut commands: Commands,
    emitters: Query<
        (
            &PxEmitterSprites,
            &PxEmitterRange,
            &PxAnchor,
            &L,
            &PxCanvas,
            &PxParticleLifetime,
            &PxVelocity,
            &PxEmitterFrequency,
            &PxEmitterFn,
            &PxEmitterSimulation,
        ),
        Added<PxEmitterSimulation>,
    >,
    time: Res<Time>,
    mut rng: ResMut<GlobalRng>,
) {
    for (
        sprites,
        range,
        anchor,
        layer,
        canvas,
        lifetime,
        velocity,
        frequency,
        on_spawn,
        simulation,
    ) in &emitters
    {
        if *simulation != PxEmitterSimulation::Simulate {
            continue;
        }

        let current_time = time.last_update().unwrap_or_else(|| time.startup()) + TIME_OFFSET;
        let mut simulated_time = current_time;

        while simulated_time + **lifetime >= current_time {
            let position = IVec2::new(
                rng.i32(range.min.x..=range.max.x),
                rng.i32(range.min.y..=range.max.y),
            )
            .as_vec2()
                + **velocity * (current_time - simulated_time).as_secs_f32();

            on_spawn(&mut commands.spawn((
                PxSpriteBundle {
                    sprite: rng.sample(sprites).unwrap().clone(),
                    position:
                        IVec2::new(position.x.round() as i32, position.y.round() as i32).into(),
                    anchor: *anchor,
                    layer: layer.clone(),
                    canvas: *canvas,
                    ..default()
                },
                PxParticleBundle {
                    position: position.into(),
                    velocity: *velocity,
                    start: simulated_time.into(),
                    lifetime: *lifetime,
                },
                Name::new("Particle"),
            )));

            // In wasm, the beginning of time is the start of the program, so we `checked_sub`
            let Some(new_time) = simulated_time
                .checked_sub((frequency.max - frequency.min).mul_f32(rng.f32()) + frequency.min)
            else {
                break;
            };
            simulated_time = new_time;
        }
    }
}

fn insert_emitter_time(
    mut commands: Commands,
    emitters: Query<Entity, Added<PxEmitterFrequency>>,
    time: Res<Time>,
    mut rng: ResMut<GlobalRng>,
) {
    for emitter in &emitters {
        commands
            .entity(emitter)
            .insert(PxEmitterStart(
                time.last_update().unwrap_or_else(|| time.startup()) + TIME_OFFSET,
            ))
            .insert(RngComponent::from(&mut rng));
    }
}

fn update_emitters<L: PxLayer>(
    mut commands: Commands,
    mut emitters: Query<(
        &PxEmitterSprites,
        &PxEmitterRange,
        &PxAnchor,
        &L,
        &PxCanvas,
        &PxParticleLifetime,
        &PxVelocity,
        &mut PxEmitterFrequency,
        &PxEmitterFn,
        &mut PxEmitterStart,
        &mut RngComponent,
    )>,
    time: Res<Time>,
) {
    for (
        sprites,
        range,
        anchor,
        layer,
        canvas,
        lifetime,
        velocity,
        mut frequency,
        on_spawn,
        mut start,
        mut rng,
    ) in &mut emitters
    {
        if time.last_update().unwrap_or_else(|| time.startup()) + TIME_OFFSET - **start
            < frequency.next(rng.get_mut())
        {
            continue;
        }

        **start += frequency.update_next(rng.get_mut());
        let position = IVec2::new(
            rng.i32(range.min.x..=range.max.x),
            rng.i32(range.min.y..=range.max.y),
        );

        on_spawn(&mut commands.spawn((
            PxSpriteBundle {
                sprite: rng.sample(sprites).unwrap().clone(),
                position: position.into(),
                anchor: *anchor,
                layer: layer.clone(),
                canvas: *canvas,
                ..default()
            },
            PxParticleBundle {
                position: position.as_vec2().into(),
                velocity: *velocity,
                start: (time.last_update().unwrap_or_else(|| time.startup()) + TIME_OFFSET).into(),
                lifetime: *lifetime,
            },
            Name::new("Particle"),
        )));
    }
}

fn despawn_particles(
    mut commands: Commands,
    particles: Query<(Entity, &PxParticleLifetime, &PxParticleStart)>,
    time: Res<Time>,
) {
    for (particle, lifetime, start) in &particles {
        if time.last_update().unwrap_or_else(|| time.startup()) + TIME_OFFSET - **start
            >= **lifetime
        {
            commands.entity(particle).despawn();
        }
    }
}

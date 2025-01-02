//! Particles and particle emitters

use std::{
    fmt::{Debug, Formatter, Result},
    time::Duration,
};

use bevy::{ecs::system::EntityCommands, utils::Instant};

use crate::{
    position::{DefaultLayer, PxLayer},
    prelude::*,
    set::PxSet,
};

// https://github.com/bevyengine/bevy/issues/8483
// In wasm, time starts at 0, so it needs an offset to represent an instant before the app started.
// If a day isn't sufficient for your use case, file an issue!
const TIME_OFFSET: Duration = Duration::from_secs(60 * 60 * 24);

pub(crate) fn plug<L: PxLayer>(app: &mut App) {
    app.add_systems(
        PostUpdate,
        (
            (
                (simulate_emitters::<L>, insert_emitter_time),
                (apply_deferred, update_emitters::<L>)
                    .chain()
                    .in_set(PxSet::UpdateEmitters),
            )
                .chain(),
            despawn_particles,
        ),
    );
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
#[derive(Debug)]
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
#[derive(Debug, Default, Eq, PartialEq)]
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

/// Creates a particle emitter
#[derive(Component)]
#[require(PxAnchor, DefaultLayer, PxCanvas, PxParticleLifetime, PxVelocity)]
pub struct PxEmitter {
    /// Possible sprites for an emitter's particles
    pub sprites: Vec<Handle<PxSpriteAsset>>,
    /// Location range for an emitter's particles
    pub range: IRect,
    /// A [`PxEmitterFrequency`]
    pub frequency: PxEmitterFrequency,
    /// A [`PxEmitterSimulation`]
    pub simulation: PxEmitterSimulation,
    /// This function is run on each particle that spawns. It is run
    /// after all of the other components are added, so you can use this to override components.
    pub on_spawn: Box<dyn Fn(&mut EntityCommands) + Send + Sync>,
}

impl Default for PxEmitter {
    fn default() -> Self {
        Self {
            sprites: Vec::new(),
            range: default(),
            frequency: default(),
            simulation: default(),
            on_spawn: Box::new(|_| ()),
        }
    }
}

impl Debug for PxEmitter {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("PxEmitter")
            .field("sprites", &self.sprites)
            .field("range", &self.range)
            .field("frequency", &self.frequency)
            .field("simulation", &self.simulation)
            .field("on_spawn", &())
            .finish()
    }
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
            &PxEmitter,
            &PxAnchor,
            &L,
            &PxCanvas,
            &PxParticleLifetime,
            &PxVelocity,
        ),
        Added<PxEmitter>,
    >,
    time: Res<Time<Real>>,
    mut rng: ResMut<GlobalRng>,
) {
    for (emitter, anchor, layer, canvas, lifetime, velocity) in &emitters {
        if emitter.simulation != PxEmitterSimulation::Simulate {
            continue;
        }

        let current_time = time.last_update().unwrap_or_else(|| time.startup()) + TIME_OFFSET;
        let mut simulated_time = current_time;

        while simulated_time + **lifetime >= current_time {
            let position = IVec2::new(
                rng.i32(emitter.range.min.x..=emitter.range.max.x),
                rng.i32(emitter.range.min.y..=emitter.range.max.y),
            )
            .as_vec2()
                + **velocity * (current_time - simulated_time).as_secs_f32();

            (emitter.on_spawn)(&mut commands.spawn((
                PxSprite(rng.sample(&emitter.sprites).unwrap().clone()),
                PxPosition::from(IVec2::new(
                    position.x.round() as i32,
                    position.y.round() as i32,
                )),
                *anchor,
                layer.clone(),
                *canvas,
                PxSubPosition::from(position),
                *velocity,
                PxParticleStart::from(simulated_time),
                *lifetime,
                Name::new("Particle"),
            )));

            // In wasm, the beginning of time is the start of the program, so we `checked_sub`
            let Some(new_time) = simulated_time.checked_sub(
                (emitter.frequency.max - emitter.frequency.min).mul_f32(rng.f32())
                    + emitter.frequency.min,
            ) else {
                break;
            };
            simulated_time = new_time;
        }
    }
}

fn insert_emitter_time(
    mut commands: Commands,
    emitters: Query<Entity, Added<PxEmitter>>,
    time: Res<Time<Real>>,
    mut rng: ResMut<GlobalRng>,
) {
    for emitter in &emitters {
        commands.entity(emitter).insert((
            PxEmitterStart(time.last_update().unwrap_or_else(|| time.startup()) + TIME_OFFSET),
            RngComponent::from(&mut rng),
        ));
    }
}

fn update_emitters<L: PxLayer>(
    mut commands: Commands,
    mut emitters: Query<(
        &mut PxEmitter,
        &PxAnchor,
        &L,
        &PxCanvas,
        &PxParticleLifetime,
        &PxVelocity,
        &mut PxEmitterStart,
        &mut RngComponent,
    )>,
    time: Res<Time<Real>>,
) {
    for (mut emitter, anchor, layer, canvas, lifetime, velocity, mut start, mut rng) in
        &mut emitters
    {
        if time.last_update().unwrap_or_else(|| time.startup()) + TIME_OFFSET - **start
            < emitter.frequency.next(rng.get_mut())
        {
            continue;
        }

        **start += emitter.frequency.update_next(rng.get_mut());
        let position = IVec2::new(
            rng.i32(emitter.range.min.x..=emitter.range.max.x),
            rng.i32(emitter.range.min.y..=emitter.range.max.y),
        );

        (emitter.on_spawn)(&mut commands.spawn((
            PxSprite(rng.sample(&emitter.sprites).unwrap().clone()),
            PxPosition::from(position),
            *anchor,
            layer.clone(),
            *canvas,
            PxSubPosition::from(position.as_vec2()),
            *velocity,
            PxParticleStart::from(
                time.last_update().unwrap_or_else(|| time.startup()) + TIME_OFFSET,
            ),
            *lifetime,
            Name::new("Particle"),
        )));
    }
}

fn despawn_particles(
    mut commands: Commands,
    particles: Query<(Entity, &PxParticleLifetime, &PxParticleStart)>,
    time: Res<Time<Real>>,
) {
    for (particle, lifetime, start) in &particles {
        if time.last_update().unwrap_or_else(|| time.startup()) + TIME_OFFSET - **start
            >= **lifetime
        {
            commands.entity(particle).despawn();
        }
    }
}

//! Contains the [`Blink`] component.

use std::time::Duration;

use bevy_derive::{Deref, DerefMut};

use crate::prelude::*;

pub(crate) fn plug(_app: &mut App) {
    #[cfg(feature = "headed")]
    _app.add_systems(PostUpdate, blink);
}

/// Toggles `Visibility` whenever the timer finishes
#[derive(Component, Deref, DerefMut)]
#[cfg_attr(feature = "headed", require(Visibility))]
pub struct Blink(Timer);

impl Blink {
    /// Creates a `Blink` with the given period
    pub fn new(period: Duration) -> Self {
        Self(Timer::new(period, TimerMode::Repeating))
    }
}

#[cfg(feature = "headed")]
fn blink(mut blinks: Query<(&mut Blink, &mut Visibility)>, time: Res<Time>) {
    for (mut blink, mut visibility) in &mut blinks {
        blink.tick(time.delta());

        if blink.just_finished() {
            visibility.toggle_inherited_hidden();
        }
    }
}

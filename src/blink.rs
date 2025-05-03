use std::time::Duration;

use crate::prelude::*;

pub(crate) fn plug(app: &mut App) {
    app.add_systems(PostUpdate, blink);
}

#[derive(Component)]
#[require(Visibility)]
pub struct Blink {
    pub timer: Timer,
}

impl Blink {
    pub fn new(period: Duration) -> Self {
        Self {
            timer: Timer::new(period, TimerMode::Repeating),
        }
    }
}

fn blink(mut blinks: Query<(&mut Blink, &mut Visibility)>, time: Res<Time>) {
    for (mut blink, mut visibility) in &mut blinks {
        blink.timer.tick(time.delta());

        if blink.timer.just_finished() {
            visibility.toggle_inherited_hidden();
        }
    }
}

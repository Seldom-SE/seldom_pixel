//! Cursor

use bevy_derive::{Deref, DerefMut};
use bevy_render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy_window::PrimaryWindow;

use crate::{
    filter::PxFilterAsset,
    prelude::*,
    screen::{screen_scale, Screen},
    set::PxSet,
};

pub(crate) fn plug(app: &mut App) {
    app.add_plugins((
        ExtractResourcePlugin::<PxCursor>::default(),
        ExtractResourcePlugin::<PxCursorPosition>::default(),
        ExtractResourcePlugin::<CursorState>::default(),
    ))
    .init_resource::<PxCursor>()
    .init_resource::<PxCursorPosition>()
    .add_systems(
        PreUpdate,
        update_cursor_position.in_set(PxSet::UpdateCursorPosition),
    )
    .add_systems(PostUpdate, change_cursor);
}

/// Resource that defines whether to use an in-game cursor
#[derive(ExtractResource, Resource, Clone, Default, Debug)]
pub enum PxCursor {
    /// Use the operating system's cursor
    #[default]
    Os,
    /// Use an in-game pixel cursor. If the cursor feels like it lags behind,
    /// consider using `bevy_framepace`.
    Filter {
        /// Filter to use when not clicking
        idle: Handle<PxFilterAsset>,
        /// Filter to use when left clicking
        left_click: Handle<PxFilterAsset>,
        /// Filter to use when right clicking
        right_click: Handle<PxFilterAsset>,
    },
}

/// Resource marking the cursor's position. Measured in pixels from the bottom-left of the screen.
/// Contains [`None`] if the cursor is off-screen. The cursor's world position
/// is the contained value plus [`PxCamera`]'s contained value.
#[derive(ExtractResource, Resource, Deref, DerefMut, Clone, Default, Debug)]
pub struct PxCursorPosition(pub Option<UVec2>);

fn update_cursor_position(
    mut move_events: EventReader<CursorMoved>,
    mut leave_events: EventReader<CursorLeft>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    screen: Res<Screen>,
    mut position: ResMut<PxCursorPosition>,
    windows: Query<&Window>,
) {
    if leave_events.read().last().is_some() {
        **position = None;
        return;
    }

    let Some(event) = move_events.read().last() else {
        return;
    };

    let Ok((camera, tf)) = cameras.single() else {
        return;
    };

    let Ok(window) = windows.single() else {
        return;
    };

    let Ok(new_position) = camera.viewport_to_world_2d(tf, event.position) else {
        **position = None;
        return;
    };

    let new_position = new_position
        / screen_scale(
            screen.computed_size,
            Vec2::new(window.width(), window.height()),
        )
        * screen.computed_size.as_vec2()
        + screen.computed_size.as_vec2() / 2.;

    **position = (new_position.cmpge(Vec2::ZERO).all()
        && new_position.cmplt(screen.computed_size.as_vec2()).all())
    .then(|| new_position.as_uvec2());
}

fn change_cursor(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    cursor: Res<PxCursor>,
    cursor_pos: Res<PxCursorPosition>,
) {
    if !cursor.is_changed() && !cursor_pos.is_changed() {
        return;
    }

    let Ok(mut window) = windows.single_mut() else {
        return;
    };

    window.cursor_options.visible = cursor_pos.is_none()
        || match *cursor {
            PxCursor::Os => true,
            PxCursor::Filter { .. } => false,
        };
}

#[derive(Resource)]
pub(crate) enum CursorState {
    Idle,
    Left,
    Right,
}

impl ExtractResource for CursorState {
    type Source = ButtonInput<MouseButton>;

    fn extract_resource(source: &ButtonInput<MouseButton>) -> Self {
        use CursorState::*;

        if source.pressed(MouseButton::Left) {
            Left
        } else if source.pressed(MouseButton::Right) {
            Right
        } else {
            Idle
        }
    }
}

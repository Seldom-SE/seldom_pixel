//! Cursor

use bevy::window::PrimaryWindow;

use crate::{
    asset::PxAsset,
    filter::PxFilterData,
    image::PxImageSliceMut,
    palette::Palette,
    prelude::*,
    screen::{Screen, ScreenMarker},
    set::PxSet,
};

pub(crate) fn plug(app: &mut App) {
    app.init_resource::<PxCursor>()
        .init_resource::<PxCursorPosition>()
        .add_systems(
            PreUpdate,
            update_cursor_position
                .run_if(resource_exists::<Palette>)
                .in_set(PxSet::UpdateCursorPosition),
        )
        .configure_sets(PostUpdate, PxSet::DrawCursor.after(PxSet::Draw))
        .add_systems(
            PostUpdate,
            (
                change_cursor.before(PxSet::DrawCursor),
                draw_cursor.in_set(PxSet::DrawCursor).in_set(PxSet::Loaded),
            ),
        );
}

/// Resource that defines whether to use an in-game cursor
#[derive(Debug, Default, Resource)]
pub enum PxCursor {
    /// Use the operating system's cursor
    #[default]
    Os,
    /// Use an in-game pixel cursor. If the cursor feels like it lags behind,
    /// consider using `bevy_framepace`.
    Filter {
        /// Filter to use when not clicking
        idle: Handle<PxFilter>,
        /// Filter to use when left clicking
        left_click: Handle<PxFilter>,
        /// Filter to use when right clicking
        right_click: Handle<PxFilter>,
    },
}

/// Resource marking the cursor's position. Measured in pixels from the bottom-left of the screen.
/// Contains [`None`] if the cursor is off-screen. The cursor's world position
/// is the contained value plus [`PxCamera`]'s contained value.
#[derive(Debug, Default, Deref, DerefMut, Resource)]
pub struct PxCursorPosition(pub Option<UVec2>);

fn update_cursor_position(
    mut move_events: EventReader<CursorMoved>,
    mut leave_events: EventReader<CursorLeft>,
    screens: Query<&Transform, With<ScreenMarker>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    screen: Res<Screen>,
    mut position: ResMut<PxCursorPosition>,
) {
    if leave_events.read().next().is_some() {
        **position = None;
        return;
    }

    let Some(event) = move_events.read().last() else {
        return;
    };
    let Ok((camera, tf)) = cameras.get_single() else {
        return;
    };

    let Some(new_position) = camera.viewport_to_world_2d(tf, event.position) else {
        **position = None;
        return;
    };
    let new_position = new_position / screens.single().scale.truncate() * screen.size.as_vec2()
        + screen.size.as_vec2() / 2.;

    **position = (new_position.cmpge(Vec2::ZERO).all()
        && new_position.cmplt(screen.size.as_vec2()).all())
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

    windows.single_mut().cursor.visible = cursor_pos.is_none()
        || match *cursor {
            PxCursor::Os => true,
            PxCursor::Filter { .. } => false,
        };
}

fn draw_cursor(
    screen: Res<Screen>,
    cursor: Res<PxCursor>,
    cursor_pos: Res<PxCursorPosition>,
    filters: Res<Assets<PxFilter>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut images: ResMut<Assets<Image>>,
) {
    if let PxCursor::Filter {
        idle,
        left_click,
        right_click,
    } = &*cursor
    {
        if let Some(cursor_pos) = **cursor_pos {
            if let Some(PxAsset::Loaded {
                asset: PxFilterData(filter),
            }) = filters.get(if mouse.pressed(MouseButton::Left) {
                left_click
            } else if mouse.pressed(MouseButton::Right) {
                right_click
            } else {
                idle
            }) {
                let mut image =
                    PxImageSliceMut::from_image_mut(images.get_mut(&screen.image).unwrap());

                if let Some(pixel) = image.get_pixel_mut(cursor_pos.as_ivec2()) {
                    *pixel = filter
                        .get_pixel(IVec2::new(*pixel as i32, 0))
                        .expect("filter is incorrect size");
                }
            }
        }
    }
}

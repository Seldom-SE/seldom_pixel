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

pub(crate) fn cursor_plugin(app: &mut App) {
    app.init_resource::<PxCursor>()
        .init_resource::<PxCursorPosition>()
        .configure_set(PxSet::UpdateCursorPosition.in_base_set(CoreSet::PreUpdate))
        .configure_set(
            PxSet::DrawCursor
                .after(PxSet::Draw)
                .in_base_set(CoreSet::PostUpdate),
        )
        .add_system(
            update_cursor_position
                .run_if(resource_exists::<Palette>())
                .in_set(PxSet::UpdateCursorPosition),
        )
        .add_system(change_cursor.before(PxSet::DrawCursor))
        .add_system(draw_cursor.in_set(PxSet::DrawCursor).in_set(PxSet::Loaded));
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
    cameras: Query<&Transform, With<Camera2d>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    screen: Res<Screen>,
    mut position: ResMut<PxCursorPosition>,
) {
    if leave_events.iter().next().is_some() {
        **position = None;
        return;
    }

    let Some(event) = move_events.iter().last() else { return };
    let Ok(camera) = cameras.get_single() else { return };
    let window = windows.single();
    let margin = (window.width() - window.height()) / 2.;

    let new_position = (camera.compute_matrix()
        * Vec4::new(
            event.position.x - margin.max(0.),
            event.position.y + margin.min(0.),
            0.,
            1.,
        ))
    .truncate()
    .truncate()
        / screens.single().scale.truncate()
        * screen.size.as_vec2();

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
    mouse: Res<Input<MouseButton>>,
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

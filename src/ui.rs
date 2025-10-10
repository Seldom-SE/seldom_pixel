//! `seldom_pixel`'s UI system. The building blocks of UI are here, but they are all just pieces.
//! For example, there is a [`PxTextField`] component, but if you spawn it on its own, the text
//! field won't have a background, and you won't even be able to type in it. Instead, you should
//! make your own helper functions that compose UI components together. For a text field, you could
//! use a [`PxStack`] with a white [`PxRect`] background and a [`PxTextField`], and add an observer
//! on [`PxRect`] that sets [`InputFocus`] to the text field.
//!
//! For more information, browse this module and see the `ui` example.

// TODO UI example
// TODO Feature parity between widgets
// TODO Split into modules

use std::time::Duration;

use bevy_derive::{Deref, DerefMut};
use bevy_ecs::system::SystemId;
#[cfg(feature = "headed")]
use bevy_input::{
    ButtonState, InputSystems,
    keyboard::{Key, KeyboardInput, NativeKey},
    mouse::MouseWheel,
};
#[cfg(feature = "headed")]
use bevy_input_focus::InputFocus;
use bevy_math::{ivec2, uvec2};

use crate::{
    blink::Blink,
    position::{DefaultLayer, Spatial},
    prelude::*,
    screen::Screen,
    set::PxSet,
};

pub(crate) fn plug<L: PxLayer>(app: &mut App) {
    #[cfg(feature = "headed")]
    app.add_systems(
        PreUpdate,
        (
            (update_key_fields, update_text_fields).run_if(resource_exists::<InputFocus>),
            scroll,
        )
            .after(InputSystems),
    )
    .add_systems(
        PostUpdate,
        (
            update_key_field_focus,
            update_text_field_focus.before(caret_blink),
        )
            .run_if(resource_exists::<InputFocus>),
    );
    app.add_systems(
        PostUpdate,
        (caret_blink, layout::<L>.before(PxSet::Picking)).chain(),
    );
}

// TODO Work on this naming

#[derive(Component)]
#[require(PxCanvas, DefaultLayer)]
pub struct PxUiRoot;

#[derive(Component, Deref, DerefMut, Default, Reflect)]
#[cfg_attr(feature = "headed", require(Visibility))]
pub struct PxMinSize(pub UVec2);

#[derive(Component, Deref, DerefMut, Reflect)]
#[cfg_attr(feature = "headed", require(Visibility))]
pub struct PxMargin(pub u32);

impl Default for PxMargin {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Component, Default, Clone)]
pub struct PxRowSlot {
    pub stretch: bool,
}

#[derive(Component, Default, Clone, Reflect)]
#[cfg_attr(feature = "headed", require(Visibility))]
pub struct PxRow {
    pub vertical: bool,
    pub space_between: u32,
}

#[derive(Default, Clone, Reflect)]
pub struct PxGridRow {
    pub stretch: bool,
}

#[derive(Default, Clone, Reflect)]
pub struct PxGridRows {
    pub rows: Vec<PxGridRow>,
    pub space_between: u32,
}

#[derive(Component, Clone)]
#[cfg_attr(feature = "headed", require(Visibility))]
pub struct PxGrid {
    pub width: u32,
    pub rows: PxGridRows,
    pub columns: PxGridRows,
}

impl Default for PxGrid {
    fn default() -> Self {
        Self {
            width: 2,
            rows: default(),
            columns: default(),
        }
    }
}

#[derive(Component, Clone, Reflect)]
#[cfg_attr(feature = "headed", require(Visibility))]
pub struct PxStack;

#[derive(Component, Default, Clone, Copy, Reflect)]
#[require(PxInvertMask, PxRect)]
pub struct PxScroll {
    pub horizontal: bool,
    pub scroll: u32,
    pub max_scroll: u32,
}

// TODO Should be modular
#[cfg(feature = "headed")]
fn scroll(mut scrolls: Query<&mut PxScroll>, mut wheels: MessageReader<MouseWheel>) {
    for wheel in wheels.read() {
        for mut scroll in &mut scrolls {
            scroll.scroll = scroll
                .scroll
                .saturating_add_signed(-wheel.y as i32)
                .min(scroll.max_scroll);
        }
    }
}

#[derive(Component, Reflect)]
#[require(PxText)]
#[reflect(from_reflect = false)]
pub struct PxKeyField {
    pub caret: char,
    /// System that creates the text label
    ///
    /// Ideally, this would accept a Bevy `Key`, but there doesn't seem to be a way to convert a
    /// winit `PhysicalKey` to a winit `Key`, so it wouldn't be possible to run this when building
    /// the UI (ie in `PxUiBuilder::dyn_insert_into`) or update all the text if the keyboard layout
    /// changes.
    #[reflect(ignore)]
    pub key_to_str: SystemId<In<KeyCode>, String>,
    pub cached_text: String,
}

#[cfg(feature = "headed")]
fn update_key_field_focus(
    mut prev_focus: Local<Option<Entity>>,
    mut fields: Query<(&PxKeyField, &mut PxText, &mut Visibility, Entity)>,
    focus: Res<InputFocus>,
    mut cmd: Commands,
) {
    let focus = focus.get();

    if *prev_focus == focus {
        return;
    }

    if let Some(prev_focus) = *prev_focus
        && let Ok((field, mut text, mut visibility, id)) = fields.get_mut(prev_focus)
    {
        text.value = field.cached_text.clone();
        *visibility = Visibility::Inherited;
        cmd.entity(id).remove::<Blink>();
    }

    if let Some(focus) = focus
        && let Ok((field, mut text, _, id)) = fields.get_mut(focus)
    {
        text.value = field.caret.to_string();
        cmd.entity(id)
            .try_insert(Blink::new(Duration::from_millis(500)));
    }

    *prev_focus = focus;
}

#[derive(EntityEvent)]
pub struct PxKeyFieldUpdate {
    pub entity: Entity,
    pub key: KeyCode,
}

// TODO Should be modular
#[cfg(feature = "headed")]
fn update_key_fields(
    mut fields: Query<Entity, With<PxKeyField>>,
    mut focus: ResMut<InputFocus>,
    mut keys: MessageReader<KeyboardInput>,
    mut cmd: Commands,
) {
    let mut keys = keys.read();
    let key = keys.find(|key| matches!(key.state, ButtonState::Pressed));
    keys.last();
    let Some(key) = key else {
        return;
    };

    let Some(focus_id) = focus.get() else {
        return;
    };

    let Ok(field_id) = fields.get_mut(focus_id) else {
        return;
    };

    let key = key.key_code;

    cmd.queue(move |world: &mut World| {
        let Some(field) = world.get::<PxKeyField>(field_id) else {
            return;
        };

        let key = match world.run_system_with(field.key_to_str, key) {
            Ok(key) => key,
            Err(err) => {
                error!("couldn't get text for pressed key for key field: {err}");
                return;
            }
        };

        if let Some(mut field) = world.get_mut::<PxKeyField>(field_id) {
            field.cached_text = key.clone();
        }

        if let Some(mut text) = world.get_mut::<PxText>(field_id) {
            text.value = key;
        };
    });

    cmd.trigger(PxKeyFieldUpdate {
        entity: field_id,
        key,
    });

    focus.clear();
}

#[derive(Reflect)]
pub struct PxCaret {
    pub state: bool,
    pub timer: Timer,
}

impl Default for PxCaret {
    fn default() -> Self {
        Self {
            state: true,
            timer: Timer::new(Duration::from_millis(500), TimerMode::Repeating),
        }
    }
}

#[derive(Component, Reflect)]
#[require(PxText)]
pub struct PxTextField {
    pub cached_text: String,
    pub caret_char: char,
    pub caret: Option<PxCaret>,
}

#[cfg(feature = "headed")]
fn update_text_field_focus(
    mut prev_focus: Local<Option<Entity>>,
    mut fields: Query<(&mut PxTextField, &mut PxText)>,
    focus: Res<InputFocus>,
) {
    let focus = focus.get();

    if *prev_focus == focus {
        return;
    }

    if let Some(prev_focus) = *prev_focus
        && let Ok((mut field, mut text)) = fields.get_mut(prev_focus)
    {
        text.value = field.cached_text.clone();
        field.caret = None;
    }

    if let Some(focus) = focus
        && let Ok((mut field, mut text)) = fields.get_mut(focus)
    {
        field.cached_text = text.value.clone();
        text.value += &field.caret_char.to_string();
        field.caret = Some(default());
    }

    *prev_focus = focus;
}

fn caret_blink(mut fields: Query<(&mut PxTextField, &mut PxText)>, time: Res<Time>) {
    for (mut field, mut text) in &mut fields {
        let Some(ref mut caret) = field.caret else {
            continue;
        };

        caret.timer.tick(time.delta());

        if caret.timer.just_finished() {
            caret.state ^= true;
            let state = caret.state;

            text.value = field.cached_text.clone();

            if state {
                text.value += &field.caret_char.to_string();
            }
        }
    }
}

#[derive(EntityEvent)]
pub struct PxTextFieldUpdate {
    pub entity: Entity,
    pub text: String,
}

// TODO Should be modular
#[cfg(feature = "headed")]
fn update_text_fields(
    mut fields: Query<(&mut PxTextField, &mut PxText)>,
    focus: Res<InputFocus>,
    mut keys: MessageReader<KeyboardInput>,
    mut cmd: Commands,
) {
    let keys = keys
        .read()
        .filter(|key| matches!(key.state, ButtonState::Pressed))
        .collect::<Vec<_>>();

    if keys.is_empty() {
        return;
    }

    let Some(focus_id) = focus.get() else {
        return;
    };

    let Ok((mut field, mut text)) = fields.get_mut(focus_id) else {
        return;
    };

    for key in keys {
        match key.logical_key {
            Key::Character(ref characters) | Key::Unidentified(NativeKey::Web(ref characters)) => {
                for character in characters.chars() {
                    field.cached_text += &character.to_string();
                }
            }
            Key::Space => field.cached_text += " ",
            Key::Backspace => {
                field.cached_text.pop();
            }
            _ => (),
        }
    }

    text.value = field.cached_text.clone() + &field.caret_char.to_string();
    field.caret = Some(default());

    cmd.trigger(PxTextFieldUpdate {
        entity: focus_id,
        text: field.cached_text.clone(),
    });
}

// If layouting ends up being too slow, make a tree of min sizes up front and lookup in that
fn calc_min_size<L: PxLayer>(
    ui: Entity,
    uis: Query<(
        AnyOf<(
            (&PxMinSize, Option<&Children>),
            (&PxMargin, Option<&Children>),
            (&PxRow, Option<&Children>),
            (&PxGrid, Option<&Children>),
            (&PxStack, Option<&Children>),
            (Option<(&PxScroll, &Children)>, &PxRect, &PxFilterLayers<L>),
            &PxSprite,
            &PxText,
        )>,
        Option<&L>,
        Option<(&PxPosition, &PxCanvas)>,
    )>,
    typefaces: &Assets<PxTypeface>,
    sprites: &Assets<PxSpriteAsset>,
) -> UVec2 {
    let Ok(((min_size, margin, row, grid, stack, rect, sprite, text), _, _)) = uis.get(ui) else {
        // This includes `PxSpace`. Surprise, the `PxSpace` component doesn't do anything at all.
        // It's just easier to spawn in UI.
        return UVec2::ZERO;
    };

    if let Some((min_size, children)) = min_size {
        return match children.map(|children| &**children) {
            None | Some([]) => **min_size,
            Some(&[content]) => {
                calc_min_size(content, uis.as_readonly(), typefaces, sprites).max(**min_size)
            }
            Some([_, _, ..]) => {
                warn!("`PxMinSize` has multiple children");
                **min_size
            }
        };
    }

    if let Some((margin, children)) = margin {
        let margin = 2 * UVec2::splat(**margin);

        return match children.map(|children| &**children) {
            None | Some([]) => margin,
            Some(&[content]) => {
                calc_min_size(content, uis.as_readonly(), typefaces, sprites) + margin
            }
            Some([_, _, ..]) => {
                warn!("`PxMargin` has multiple children");
                margin
            }
        };
    }

    fn dim(vec: UVec2, y: bool) -> u32 {
        if y { vec.y } else { vec.x }
    }

    fn dim_mut(vec: &mut UVec2, y: bool) -> &mut u32 {
        if y { &mut vec.y } else { &mut vec.x }
    }

    if let Some((row, children)) = row {
        let vert = row.vertical;
        let mut size = UVec2::ZERO;

        let children = if let Some(children) = children {
            &**children
        } else {
            &[]
        };

        *dim_mut(&mut size, vert) += children.len().saturating_sub(1) as u32 * row.space_between;

        for &entry in children {
            let min_size = calc_min_size(entry, uis.as_readonly(), typefaces, sprites);

            *dim_mut(&mut size, vert) += dim(min_size, vert);

            let cross_size = dim(min_size, !vert);
            if cross_size > dim(size, !vert) {
                *dim_mut(&mut size, !vert) = cross_size;
            }
        }

        return size;
    }

    if let Some((grid, children)) = grid {
        let mut column_widths = vec![0; grid.width as usize];
        let children = if let Some(children) = children {
            &**children
        } else {
            &[]
        };
        let mut height = (children.len() as u32)
            .div_ceil(grid.width)
            .saturating_sub(1)
            * grid.rows.space_between;

        for row in children.chunks(grid.width as usize) {
            let mut row_height = 0;

            for (column, &entry) in row.iter().enumerate() {
                let size = calc_min_size(entry, uis.as_readonly(), typefaces, sprites);

                if size.x > column_widths[column] {
                    column_widths[column] = size.x;
                }

                if size.y > row_height {
                    row_height = size.y;
                }
            }

            height += row_height;
        }

        return uvec2(
            column_widths.into_iter().sum::<u32>()
                + grid.width.saturating_sub(1) * grid.columns.space_between,
            height,
        );
    }

    if let Some((_, children)) = stack {
        let mut size = UVec2::ZERO;

        for &entry in children.iter().flat_map(|children| &***children) {
            size = size.max(calc_min_size(entry, uis.as_readonly(), typefaces, sprites));
        }

        return size;
    }

    if let Some((scroll, _, _)) = rect {
        let Some((scroll, children)) = scroll else {
            return UVec2::ZERO;
        };

        let mut children = children.iter();

        let (mut size, bar_size) = if let Some(content) = children.next() {
            (
                calc_min_size(content, uis.as_readonly(), typefaces, sprites),
                if let Some(bar) = children.next() {
                    calc_min_size(bar, uis.as_readonly(), typefaces, sprites).max(
                        if let Some(bar_bg) = children.next() {
                            calc_min_size(bar_bg, uis.as_readonly(), typefaces, sprites)
                        } else {
                            UVec2::ZERO
                        },
                    )
                } else {
                    UVec2::ZERO
                },
            )
        } else {
            default()
        };

        if children.next().is_some() {
            warn!("`PxScroll` has more than 3 children");
        }

        let horz = scroll.horizontal;

        *dim_mut(&mut size, horz) += dim(bar_size, horz);
        let bar_main = dim(bar_size, !horz);
        if bar_main > dim(size, !horz) {
            *dim_mut(&mut size, !horz) = bar_main;
        }

        return size;
    }

    if let Some(sprite) = sprite {
        return if let Some(sprite) = sprites.get(&**sprite) {
            sprite.frame_size()
        } else {
            UVec2::ZERO
        };
    }

    if let Some(text) = text {
        let Some(typeface) = typefaces.get(&text.typeface) else {
            return UVec2::ZERO;
        };

        return uvec2(
            text.value
                .chars()
                .map(|char| {
                    if let Some(char) = typeface.characters.get(&char) {
                        char.frame_size().x + 1
                    } else if let Some(separator) = typeface.separators.get(&char) {
                        separator.width
                    } else {
                        error!(r#"character "{char}" in text isn't in typeface"#);
                        0
                    }
                })
                .sum::<u32>()
                .saturating_sub(1),
            typeface.height,
        );
    }

    unreachable!()
}

fn layout_inner<L: PxLayer>(
    target_rect: IRect,
    target_layer: &L,
    target_canvas: PxCanvas,
    ui: Entity,
    mut uis: Query<(
        AnyOf<(
            (&PxMinSize, Option<&Children>),
            (&PxMargin, Option<&Children>),
            (&PxRow, Option<&Children>),
            (&PxGrid, Option<&Children>),
            (&PxStack, Option<&Children>),
            (
                Option<(&mut PxScroll, &Children)>,
                &mut PxRect,
                &mut PxFilterLayers<L>,
            ),
            &PxSprite,
            &mut PxText,
        )>,
        Option<&mut L>,
        Option<(&mut PxPosition, &mut PxCanvas)>,
    )>,
    row_slots: Query<&PxRowSlot>,
    typefaces: &Assets<PxTypeface>,
    sprites: &Assets<PxSpriteAsset>,
) -> Result<Option<L>> {
    let Ok(((min_size, margin, row, grid, stack, rect, sprite, text), _, _)) = uis.get(ui) else {
        return Ok(None);
    };

    if let Some((_, children)) = min_size {
        return match children.map(|children| &**children) {
            None | Some([]) => Ok(None),
            Some(&[content]) => layout_inner(
                target_rect,
                target_layer,
                target_canvas,
                content,
                uis,
                row_slots,
                typefaces,
                sprites,
            ),
            Some([_, _, ..]) => {
                warn!("`PxMinSize` has multiple children");
                Ok(None)
            }
        };
    }

    if let Some((margin, children)) = margin {
        return match children.map(|children| &**children) {
            None | Some([]) => Ok(None),
            Some(&[content]) => layout_inner(
                IRect {
                    min: target_rect.min + **margin as i32,
                    max: target_rect.max - **margin as i32,
                },
                target_layer,
                target_canvas,
                content,
                uis,
                row_slots,
                typefaces,
                sprites,
            ),
            Some([_, _, ..]) => {
                warn!("`PxMargin` has multiple children");
                Ok(None)
            }
        };
    }

    fn dim(vec: IVec2, y: bool) -> i32 {
        if y { vec.y } else { vec.x }
    }

    // Adds to x, subtracts from y
    fn add(augend: i32, addend: i32, y: bool) -> i32 {
        if y { augend - addend } else { augend + addend }
    }

    fn rect_size(rect: IRect, y: bool) -> i32 {
        if y { rect.height() } else { rect.width() }
    }

    if let Some((row, children)) = row {
        let row = row.clone();
        let children = children
            .iter()
            .flat_map(|children| &**children)
            .copied()
            .collect::<Vec<_>>();

        fn dim_mut(vec: &mut IVec2, y: bool) -> &mut i32 {
            if y { &mut vec.y } else { &mut vec.x }
        }

        let vert = row.vertical;
        let mut pos = ivec2(target_rect.min.x, target_rect.max.y);
        let mut remaining_stretchers = children
            .iter()
            .map(|&entry| row_slots.get(entry).cloned().unwrap_or_default())
            .filter(|slot| slot.stretch)
            .count() as i32;
        let mut stretch_budget = rect_size(target_rect, vert)
            - dim(
                calc_min_size(ui, uis.as_readonly(), typefaces, sprites).as_ivec2(),
                vert,
            );
        let fill_size = rect_size(target_rect, !vert);

        let mut layer = None::<L>;

        for &child in &children {
            let slot = row_slots.get(child).cloned().unwrap_or_default();
            let mut size = calc_min_size(child, uis.as_readonly(), typefaces, sprites).as_ivec2();
            if slot.stretch {
                // For simplicity, we just split the extra size among the stretched entries evenly
                // instead of prioritizing the smallest. I might change this in the future.
                let extra_size = stretch_budget / remaining_stretchers;
                *dim_mut(&mut size, vert) += extra_size;
                stretch_budget -= extra_size;
                remaining_stretchers -= 1;
            }

            // if entry.fill {
            *dim_mut(&mut size, !vert) = fill_size;
            // }

            let entry_layer = if let Some(ref layer) = layer {
                layer.clone().next().unwrap_or(layer.clone())
            } else {
                target_layer.clone()
            };

            // Improvements to the layouting could make it so that most things can share layers
            if let Some(last_layer) = layout_inner(
                IRect {
                    min: ivec2(pos.x, pos.y - size.y),
                    max: ivec2(pos.x + size.x, pos.y),
                },
                &entry_layer,
                target_canvas,
                child,
                uis.reborrow(),
                row_slots.as_readonly(),
                typefaces,
                sprites,
            )? {
                layer = Some(last_layer);
            }

            *dim_mut(&mut pos, vert) = add(
                dim(pos, vert),
                dim(size, vert) + row.space_between as i32,
                vert,
            );
        }

        return Ok(layer);
    }

    if let Some((grid, children)) = grid {
        let grid = grid.clone();
        let children = children
            .iter()
            .flat_map(|children| &**children)
            .copied()
            .collect::<Vec<_>>();

        let mut column_widths = vec![0; grid.width as usize];
        let mut row_heights = vec![0; children.len().div_ceil(grid.width as usize)];

        for (row_index, row) in children.chunks(grid.width as usize).enumerate() {
            for (column, &entry) in row.iter().enumerate() {
                let size = calc_min_size(entry, uis.as_readonly(), typefaces, sprites).as_ivec2();

                if size.x > column_widths[column] {
                    column_widths[column] = size.x;
                }

                if size.y > row_heights[row_index] {
                    row_heights[row_index] = size.y;
                }
            }
        }

        let min_size = calc_min_size(ui, uis.as_readonly(), typefaces, sprites).as_ivec2();

        let mut remaining_stretching_rows =
            grid.rows.rows.iter().filter(|row| row.stretch).count() as i32;
        let mut row_stretch_budget = target_rect.height() - min_size.y;

        for (index, row) in grid.rows.rows.iter().enumerate() {
            if index >= row_heights.len() {
                continue;
            }

            if row.stretch {
                let extra_size = row_stretch_budget / remaining_stretching_rows;
                row_heights[index] += extra_size;
                row_stretch_budget -= extra_size;
                remaining_stretching_rows -= 1;
            }
        }

        let mut remaining_stretching_columns = grid
            .columns
            .rows
            .iter()
            .filter(|column| column.stretch)
            .count() as i32;
        let mut column_stretch_budget = target_rect.width() - min_size.x;

        for (index, column) in grid.columns.rows.iter().enumerate() {
            if index >= column_widths.len() {
                continue;
            }

            if column.stretch {
                let extra_size = column_stretch_budget / remaining_stretching_columns;
                column_widths[index] += extra_size;
                column_stretch_budget -= extra_size;
                remaining_stretching_columns -= 1;
            }
        }

        let mut y_pos = target_rect.max.y;

        let mut layer = None::<L>;

        for (row_index, row) in children.chunks(grid.width as usize).enumerate() {
            let mut x_pos = target_rect.min.x;
            let height = row_heights[row_index];

            for (column, &entry) in row.iter().enumerate() {
                let width = column_widths[column];

                let entry_layer = if let Some(ref layer) = layer {
                    layer.clone().next().unwrap_or(layer.clone())
                } else {
                    target_layer.clone()
                };

                if let Some(last_layer) = layout_inner(
                    IRect {
                        min: ivec2(x_pos, y_pos - height),
                        max: ivec2(x_pos + width, y_pos),
                    },
                    &entry_layer,
                    target_canvas,
                    entry,
                    uis.reborrow(),
                    row_slots.as_readonly(),
                    typefaces,
                    sprites,
                )? {
                    layer = Some(last_layer);
                };

                x_pos += width + grid.columns.space_between as i32;
            }

            y_pos -= height + grid.columns.space_between as i32;
        }

        return Ok(layer);
    }

    if let Some((_, children)) = stack {
        let children = children
            .iter()
            .flat_map(|children| &**children)
            .copied()
            .collect::<Vec<_>>();

        let mut layer = None::<L>;

        for &entry in &children {
            let entry_layer = if let Some(ref layer) = layer {
                layer.clone().next().unwrap_or(layer.clone())
            } else {
                target_layer.clone()
            };

            if let Some(last_layer) = layout_inner(
                target_rect,
                &entry_layer,
                target_canvas,
                entry,
                uis.reborrow(),
                row_slots.as_readonly(),
                typefaces,
                sprites,
            )? {
                layer = Some(last_layer);
            };
        }

        return Ok(layer);
    }

    if rect.is_some() {
        let ((_, _, _, _, _, rect, _, _), _, mut pos) = uis.get_mut(ui).unwrap();

        if let Some((_, ref mut canvas)) = pos {
            **canvas = target_canvas;
        }

        let (scroll, mut rect, mut layers) = rect.unwrap();

        if let Some((scroll, children)) = scroll {
            fn rect_start(rect: IRect, y: bool) -> i32 {
                if y { rect.max.y } else { rect.min.x }
            }

            fn rect_start_mut(rect: &mut IRect, y: bool) -> &mut i32 {
                if y { &mut rect.max.y } else { &mut rect.min.x }
            }

            fn rect_end(rect: IRect, y: bool) -> i32 {
                if y { rect.min.y } else { rect.max.x }
            }

            fn rect_end_mut(rect: &mut IRect, y: bool) -> &mut i32 {
                if y { &mut rect.min.y } else { &mut rect.max.x }
            }

            let scroll = *scroll;
            let content = children[0];
            let bar = children.get(1).copied();
            let bg = children.get(2).copied();
            if children.get(3).is_some() {
                warn!("`PxScroll` has more than 3 children");
                return Ok(None);
            }
            let horz = scroll.horizontal;

            let content_min_size =
                calc_min_size(content, uis.as_readonly(), typefaces, sprites).as_ivec2();

            let bar_min_size = if let Some(bar) = bar {
                calc_min_size(bar, uis.as_readonly(), typefaces, sprites).max(
                    if let Some(bg) = bg {
                        calc_min_size(bg, uis.as_readonly(), typefaces, sprites)
                    } else {
                        UVec2::ZERO
                    },
                )
            } else {
                UVec2::ZERO
            }
            .as_ivec2();

            let mut view_rect = target_rect;
            *rect_end_mut(&mut view_rect, horz) =
                add(rect_end(view_rect, horz), -dim(bar_min_size, horz), horz);

            let ((_, _, _, _, _, rect, _, _), _, pos) = uis.get_mut(ui).unwrap();
            let (_, mut rect, _) = rect.unwrap();
            **rect = view_rect.size().as_uvec2();
            if let Some((mut pos, _)) = pos {
                **pos = view_rect.center();
            }

            let mut content_rect = view_rect;
            *rect_start_mut(&mut content_rect, !horz) = add(
                rect_start(content_rect, !horz),
                -(scroll.scroll as i32),
                !horz,
            );
            *rect_end_mut(&mut content_rect, !horz) = add(
                rect_start(content_rect, !horz),
                dim(content_min_size, !horz),
                !horz,
            );

            let mut layer = None;

            // TODO Need to make containers with multiple entries put entries beyond the first on
            // different layers
            let last_content_layer = layout_inner(
                content_rect,
                target_layer,
                target_canvas,
                content,
                uis.reborrow(),
                row_slots.as_readonly(),
                typefaces,
                sprites,
            )?;

            let ((_, _, _, _, _, rect, _, _), _, _) = uis.get_mut(ui).unwrap();
            let (_, _, mut layers) = rect.unwrap();

            let bg_layer;
            (*layers, bg_layer) = if let Some(last_content_layer) = last_content_layer {
                layer = Some(last_content_layer.clone());

                (
                    PxFilterLayers::Range(target_layer.clone()..=last_content_layer.clone()),
                    last_content_layer
                        .clone()
                        .next()
                        .unwrap_or(last_content_layer),
                )
            } else {
                (PxFilterLayers::Many(Vec::new()), target_layer.clone())
            };

            let mut bar_rect = target_rect;
            *rect_start_mut(&mut bar_rect, horz) = rect_end(view_rect, horz);

            let last_bg_layer = bg
                .map(|bg| {
                    layout_inner(
                        bar_rect,
                        &bg_layer,
                        target_canvas,
                        bg,
                        uis.reborrow(),
                        row_slots.as_readonly(),
                        typefaces,
                        sprites,
                    )
                })
                .transpose()?
                .flatten();
            let bar_layer = if let Some(last_bg_layer) = last_bg_layer {
                layer = Some(last_bg_layer.clone());
                last_bg_layer.clone().next().unwrap_or(last_bg_layer)
            } else {
                bg_layer.clone()
            };

            let content_size = rect_size(content_rect, !horz);
            let view_size = rect_size(view_rect, !horz);
            let ratio = if content_size == 0 {
                0.
            } else {
                view_size as f32 / content_size as f32
            };
            *rect_start_mut(&mut bar_rect, !horz) = add(
                rect_start(view_rect, !horz),
                (scroll.scroll as f32 * ratio) as i32,
                !horz,
            );
            *rect_end_mut(&mut bar_rect, !horz) = add(
                rect_start(view_rect, !horz),
                ((view_size + scroll.scroll as i32) as f32 * ratio) as i32,
                !horz,
            );

            let ((_, _, _, _, _, rect, _, _), _, _) = uis.get_mut(ui).unwrap();
            let (scroll, _, _) = rect.unwrap();
            let (mut scroll, _) = scroll.unwrap();

            scroll.max_scroll = (view_size as f32 * (1. / ratio - 1.)).ceil() as u32;

            if let Some(last_bar_layer) = bar
                .map(|bar| {
                    layout_inner(
                        bar_rect,
                        &bar_layer,
                        target_canvas,
                        bar,
                        uis.reborrow(),
                        row_slots.as_readonly(),
                        typefaces,
                        sprites,
                    )
                })
                .transpose()?
                .flatten()
            {
                layer = Some(last_bar_layer);
            }

            return Ok(layer);
        } else {
            if let Some((mut pos, _)) = pos {
                **pos = target_rect.center();
            }

            let rect_layer = target_layer.clone();
            match *layers {
                PxFilterLayers::Single { ref mut layer, .. } => *layer = rect_layer,
                PxFilterLayers::Range(ref mut layers) => {
                    *layers = layers.start().clone()..=rect_layer
                }
                ref mut layers @ PxFilterLayers::Many(_) => {
                    *layers = PxFilterLayers::single_over(rect_layer)
                }
            }

            **rect = target_rect.size().as_uvec2();

            return Ok(Some(target_layer.clone()));
        }
    }

    if sprite.is_some() {
        let (_, layer, pos) = uis.get_mut(ui).unwrap();

        if let Some((mut pos, mut canvas)) = pos {
            **pos = target_rect.center();
            *canvas = target_canvas;
        }

        if let Some(mut layer) = layer {
            *layer = target_layer.clone();
        }

        return Ok(Some(target_layer.clone()));
    }

    if text.is_some() {
        let ((_, _, _, _, _, _, _, text), layer, pos) = uis.get_mut(ui).unwrap();

        if let Some(mut layer) = layer {
            *layer = target_layer.clone();
        }

        let Some((mut pos, mut canvas)) = pos else {
            return Ok(Some(target_layer.clone()));
        };

        *canvas = target_canvas;

        let mut text = text.unwrap();
        let PxText {
            ref mut value,
            ref typeface,
            ref mut line_breaks,
        } = *text;

        let Some(typeface) = typefaces.get(typeface) else {
            return Ok(Some(target_layer.clone()));
        };

        line_breaks.clear();

        let max_width = target_rect.width();
        let mut x = 0;
        let mut max_x = 0;
        let mut last_separator = None;

        for (index, char) in value.chars().enumerate() {
            let index = index as u32;

            if let Some(char) = typeface.characters.get(&char) {
                let split = x > max_width;
                if split {
                    x = 0;
                    line_breaks.push(last_separator.unwrap_or(index.saturating_sub(1)));
                    last_separator = None;
                }

                let width = char.frame_size().x as i32;

                if x != 0 {
                    x += 1
                }
                x += width;

                if x > max_width && !split {
                    x = width;
                    line_breaks.push(last_separator.unwrap_or(index.saturating_sub(1)));
                    last_separator = None;
                }

                if x > max_x {
                    max_x = x;
                }
            } else if let Some(separator) = typeface.separators.get(&char) {
                x += separator.width as i32;
                last_separator = Some(index);
            } else {
                error!(r#"character "{char}" in text isn't in typeface"#);
            }
        }

        let line_break_count = line_breaks.len() as i32;
        **pos = ivec2(target_rect.min.x, target_rect.max.y)
            + ivec2(
                max_x,
                -((line_break_count + 1) * typeface.height as i32 + line_break_count),
            ) / 2;

        return Ok(Some(target_layer.clone()));
    }

    unreachable!();
}

fn layout<L: PxLayer>(
    mut uis: ParamSet<(
        Query<(&L, &PxCanvas, Entity), With<PxUiRoot>>,
        Query<(
            AnyOf<(
                (&PxMinSize, Option<&Children>),
                (&PxMargin, Option<&Children>),
                (&PxRow, Option<&Children>),
                (&PxGrid, Option<&Children>),
                (&PxStack, Option<&Children>),
                (
                    Option<(&mut PxScroll, &Children)>,
                    &mut PxRect,
                    &mut PxFilterLayers<L>,
                ),
                &PxSprite,
                &mut PxText,
            )>,
            Option<&mut L>,
            Option<(&mut PxPosition, &mut PxCanvas)>,
        )>,
    )>,
    row_slots: Query<&PxRowSlot>,
    typefaces: Res<Assets<PxTypeface>>,
    sprites: Res<Assets<PxSpriteAsset>>,
    screen: Res<Screen>,
) -> Result {
    for (layer, canvas, root) in uis
        .p0()
        .iter()
        .map(|(layer, &canvas, entity)| (layer.clone(), canvas, entity))
        .collect::<Vec<_>>()
    {
        layout_inner(
            IRect {
                min: IVec2::ZERO,
                max: screen.computed_size.as_ivec2(),
            },
            &layer,
            canvas,
            root,
            uis.p1(),
            row_slots.as_readonly(),
            &typefaces,
            &sprites,
        )?;
    }

    OK
}

use crate::{cursor::PxCursorPosition, math::RectExt, prelude::*, set::PxSet};

pub(crate) fn plug(app: &mut App) {
    app.init_resource::<PxEnableButtons>()
        .add_systems(
            PreUpdate,
            interact_buttons
                .run_if(resource_equals(PxEnableButtons(true)))
                .after(PxSet::UpdateCursorPosition),
        )
        .configure_sets(
            PostUpdate,
            (
                PxSet::AddButtonAssets.run_if(resource_equals(PxEnableButtons(true))),
                PxSet::UpdateButtonAssets,
            ),
        )
        .add_systems(
            PostUpdate,
            (
                (add_button_sprites, add_button_filters).in_set(PxSet::AddButtonAssets),
                apply_deferred
                    .after(PxSet::AddButtonAssets)
                    .before(PxSet::UpdateButtonAssets),
                (update_button_sprites, update_button_filters).in_set(PxSet::UpdateButtonAssets),
                disable_buttons
                    .run_if(resource_changed::<PxEnableButtons>)
                    .run_if(resource_equals(PxEnableButtons(false))),
            ),
        );
}

/// Defines the interactable bounds for a sprite. Shares an anchor with the sprite.
/// Add to any sprite to make it a button.
#[derive(Component, Debug)]
pub struct PxInteractBounds {
    /// Size of the bounds
    pub size: UVec2,
    /// Offset from the sprite's anchor
    pub offset: UVec2,
}

impl Default for PxInteractBounds {
    fn default() -> Self {
        UVec2::ONE.into()
    }
}

impl From<UVec2> for PxInteractBounds {
    fn from(size: UVec2) -> Self {
        Self {
            size,
            offset: UVec2::ZERO,
        }
    }
}

/// Sprite to use when the button is not being interacted
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct PxIdleSprite(pub Handle<PxSprite>);

impl From<Handle<PxSprite>> for PxIdleSprite {
    fn from(image: Handle<PxSprite>) -> Self {
        Self(image)
    }
}

/// Sprite to use when the button is hovered
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct PxHoverSprite(pub Handle<PxSprite>);

impl From<Handle<PxSprite>> for PxHoverSprite {
    fn from(image: Handle<PxSprite>) -> Self {
        Self(image)
    }
}

/// Sprite to use when the button is clicked
#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct PxClickSprite(pub Handle<PxSprite>);

impl From<Handle<PxSprite>> for PxClickSprite {
    fn from(image: Handle<PxSprite>) -> Self {
        Self(image)
    }
}

/// Makes a sprite a button that changes sprite based on interaction
#[derive(Bundle, Debug, Default)]
pub struct PxButtonSpriteBundle {
    /// A [`PxInteractBounds`] component
    pub bounds: PxInteractBounds,
    /// A [`PxIdleSprite`] component
    pub idle: PxIdleSprite,
    /// A [`PxHoverSprite`] component
    pub hover: PxHoverSprite,
    /// A [`PxClickSprite`] component
    pub click: PxClickSprite,
}

/// Filter to use when the button is not being interacted
#[derive(Component, Debug, Deref, DerefMut)]
pub struct PxIdleFilter(pub Handle<PxFilter>);

impl From<Handle<PxFilter>> for PxIdleFilter {
    fn from(image: Handle<PxFilter>) -> Self {
        Self(image)
    }
}

/// Filter to use when the button is hovered
#[derive(Component, Debug, Deref, DerefMut)]
pub struct PxHoverFilter(pub Handle<PxFilter>);

impl From<Handle<PxFilter>> for PxHoverFilter {
    fn from(image: Handle<PxFilter>) -> Self {
        Self(image)
    }
}

/// Filter to use when the button is clicked
#[derive(Component, Debug, Deref, DerefMut)]
pub struct PxClickFilter(pub Handle<PxFilter>);

impl From<Handle<PxFilter>> for PxClickFilter {
    fn from(image: Handle<PxFilter>) -> Self {
        Self(image)
    }
}

/// Makes a sprite a button that changes filter based on interaction
#[derive(Bundle, Debug)]
pub struct PxButtonFilterBundle {
    /// A [`PxInteractBounds`] component
    pub bounds: PxInteractBounds,
    /// A [`PxIdleFilter`] component
    pub idle: PxIdleFilter,
    /// A [`PxHoverFilter`] component
    pub hover: PxHoverFilter,
    /// A [`PxClickFilter`] component
    pub click: PxClickFilter,
}

/// Marks a button that is being hovered
#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct PxHover;

/// Marks a button that is being clicked. Always appears with [`PxHover`]
#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct PxClick;

/// Resource that determines whether buttons are enabled
#[derive(Debug, Deref, DerefMut, PartialEq, Resource)]
pub struct PxEnableButtons(pub bool);

impl Default for PxEnableButtons {
    fn default() -> Self {
        Self(true)
    }
}

fn interact_buttons(
    mut commands: Commands,
    buttons: Query<(
        Entity,
        &PxPosition,
        &PxInteractBounds,
        &PxAnchor,
        &PxCanvas,
        Option<&PxHover>,
        Option<&PxClick>,
    )>,
    cursor_pos: Res<PxCursorPosition>,
    mouse: Res<ButtonInput<MouseButton>>,
    camera: Res<PxCamera>,
) {
    for (button, position, bounds, anchor, canvas, hovered, clicked) in &buttons {
        let mut button = commands.entity(button);

        if let Some(cursor_pos) = **cursor_pos {
            let cursor_pos = cursor_pos.as_ivec2();
            let cursor_pos = match canvas {
                PxCanvas::World => cursor_pos + **camera,
                PxCanvas::Camera => cursor_pos,
            };

            if IRect::pos_size_anchor(**position, bounds.size, *anchor)
                .contains_exclusive(cursor_pos - bounds.offset.as_ivec2())
            {
                if hovered.is_none() {
                    button.insert(PxHover);
                }

                if mouse.pressed(MouseButton::Left) {
                    if clicked.is_none() {
                        button.insert(PxClick);
                    }
                } else if clicked.is_some() {
                    button.remove::<PxClick>();
                }

                continue;
            }
        }

        if hovered.is_some() {
            button.remove::<PxHover>();
        }

        if clicked.is_some() {
            button.remove::<PxClick>();
        }
    }
}

fn disable_buttons(
    mut commands: Commands,
    hovered_buttons: Query<Entity, With<PxHover>>,
    clicked_buttons: Query<Entity, With<PxClick>>,
) {
    for button in &hovered_buttons {
        commands.entity(button).remove::<PxHover>();
    }

    for button in &clicked_buttons {
        commands.entity(button).remove::<PxClick>();
    }
}

fn add_button_sprites(
    mut commands: Commands,
    buttons: Query<(Entity, &PxIdleSprite), Added<PxIdleSprite>>,
) {
    for (button, idle) in &buttons {
        // `PxIdleSprite` derefs to a `Handle<PxSprite>`
        commands.entity(button).insert((**idle).clone());
    }
}

fn update_button_sprites(
    mut idle_buttons: Query<
        (&mut Handle<PxSprite>, &PxIdleSprite),
        (Without<PxHover>, Without<PxClick>),
    >,
    mut hovered_buttons: Query<
        (&mut Handle<PxSprite>, &PxHoverSprite),
        (With<PxHover>, Without<PxClick>),
    >,
    mut clicked_buttons: Query<
        (&mut Handle<PxSprite>, &PxClickSprite),
        (With<PxHover>, With<PxClick>),
    >,
) {
    for (mut sprite, idle_sprite) in &mut idle_buttons {
        *sprite = (*idle_sprite).clone();
    }

    for (mut sprite, hovered_sprite) in &mut hovered_buttons {
        *sprite = (*hovered_sprite).clone();
    }

    for (mut sprite, clicked_sprite) in &mut clicked_buttons {
        *sprite = (*clicked_sprite).clone();
    }
}

fn add_button_filters(
    mut commands: Commands,
    buttons: Query<(Entity, &PxIdleFilter), Added<PxIdleFilter>>,
) {
    for (button, idle) in &buttons {
        // `PxIdleFilter` derefs to a `Handle<PxFilter>`
        commands.entity(button).insert((**idle).clone());
    }
}

fn update_button_filters(
    mut idle_buttons: Query<
        (&mut Handle<PxFilter>, &PxIdleFilter),
        (Without<PxHover>, Without<PxClick>),
    >,
    mut hovered_buttons: Query<
        (&mut Handle<PxFilter>, &PxHoverFilter),
        (With<PxHover>, Without<PxClick>),
    >,
    mut clicked_buttons: Query<
        (&mut Handle<PxFilter>, &PxClickFilter),
        (With<PxHover>, With<PxClick>),
    >,
) {
    for (mut filter, idle_filter) in &mut idle_buttons {
        *filter = (*idle_filter).clone();
    }

    for (mut filter, hovered_filter) in &mut hovered_buttons {
        *filter = (*hovered_filter).clone();
    }

    for (mut filter, clicked_filter) in &mut clicked_buttons {
        *filter = (*clicked_filter).clone();
    }
}

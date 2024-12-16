use crate::{
    cursor::PxCursorPosition, filter::PxFilterAsset, math::RectExt, prelude::*, set::PxSet,
    sprite::PxSpriteAsset,
};

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

/// Makes a sprite a button that changes sprite based on interaction
#[derive(Component, Debug)]
#[require(PxSprite, PxInteractBounds)]
pub struct PxButtonSprite {
    /// Sprite to use when the button is not being interacted
    pub idle: Handle<PxSpriteAsset>,
    /// Sprite to use when the button is hovered
    pub hover: Handle<PxSpriteAsset>,
    /// Sprite to use when the button is clicked
    pub click: Handle<PxSpriteAsset>,
}

/// Makes a sprite a button that changes filter based on interaction
#[derive(Component, Debug)]
#[require(PxSprite, PxInteractBounds)]
pub struct PxButtonFilter {
    /// Filter to use when the button is not being interacted
    pub idle: Handle<PxFilterAsset>,
    /// Filter to use when the button is hovered
    pub hover: Handle<PxFilterAsset>,
    /// Filter to use when the button is clicked
    pub click: Handle<PxFilterAsset>,
}

impl Default for PxButtonFilter {
    fn default() -> Self {
        Self {
            idle: default(),
            hover: default(),
            click: default(),
        }
    }
}

// TODO Migrate to observers

/// Marks a button that is being hovered
#[derive(Component, Debug)]
pub struct PxHover;

/// Marks a button that is being clicked. Always appears with [`PxHover`]
#[derive(Component, Debug)]
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
    buttons: Query<(Entity, &PxButtonSprite), Added<PxButtonSprite>>,
) {
    for (id, button) in &buttons {
        commands.entity(id).insert(PxSprite(button.idle.clone()));
    }
}

fn update_button_sprites(
    mut idle_buttons: Query<(&mut PxSprite, &PxButtonSprite), (Without<PxHover>, Without<PxClick>)>,
    mut hovered_buttons: Query<(&mut PxSprite, &PxButtonSprite), (With<PxHover>, Without<PxClick>)>,
    mut clicked_buttons: Query<(&mut PxSprite, &PxButtonSprite), (With<PxHover>, With<PxClick>)>,
) {
    for (mut sprite, button) in &mut idle_buttons {
        **sprite = button.idle.clone();
    }

    for (mut sprite, button) in &mut hovered_buttons {
        **sprite = button.hover.clone();
    }

    for (mut sprite, button) in &mut clicked_buttons {
        **sprite = button.click.clone();
    }
}

fn add_button_filters(
    mut commands: Commands,
    buttons: Query<(Entity, &PxButtonFilter), Added<PxButtonFilter>>,
) {
    for (id, button) in &buttons {
        commands.entity(id).insert(PxFilter(button.idle.clone()));
    }
}

fn update_button_filters(
    mut idle_buttons: Query<(&mut PxFilter, &PxButtonFilter), (Without<PxHover>, Without<PxClick>)>,
    mut hovered_buttons: Query<(&mut PxFilter, &PxButtonFilter), (With<PxHover>, Without<PxClick>)>,
    mut clicked_buttons: Query<(&mut PxFilter, &PxButtonFilter), (With<PxHover>, With<PxClick>)>,
) {
    for (mut filter, button) in &mut idle_buttons {
        **filter = button.idle.clone();
    }

    for (mut filter, button) in &mut hovered_buttons {
        **filter = button.hover.clone();
    }

    for (mut filter, button) in &mut clicked_buttons {
        **filter = button.click.clone();
    }
}

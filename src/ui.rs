//! `seldom_pixel`'s UI system. The core building blocks of UI are here, but they are bare-bones.
//! For example, there is a [`PxTextField`] component, but if you spawn it on its own, the text
//! field won't have a background, and you won't even be able to type in it. Instead, you should
//! make your own helper functions that compose UI builders together. For a text field, you could
//! use a [`PxStack`] with a white [`PxRect`] background and a [`PxTextField`], and add an observer
//! on [`PxRect`] that sets [`Focus`] to the text field.
//!
//! The UI is bare-bones by design; `seldom_pixel` isn't meant to go far beyond the scope of a
//! graphics library. It means you're forced to make all of the decisions about the appearance and
//! behavior of your game's UI, for better and worse. The initial cost of this UI is a bit higher
//! than ideal. Make yourself some helpers and stay organized, and it won't be too bad.
//!
//! For more information, browse this module and see the `ui` example.

// TODO UI example
// TODO Feature parity between widgets

use std::time::Duration;

use bevy::{
    a11y::Focus,
    ecs::system::{IntoObserverSystem, SystemId},
    input::{
        keyboard::{Key, KeyboardInput, NativeKey},
        ButtonState, InputSystem,
    },
    math::{ivec2, uvec2},
};

use crate::{
    blink::Blink, filter::DefaultPxFilterLayers, position::Spatial, prelude::*, screen::Screen,
    set::PxSet,
};

pub(crate) fn plug<L: PxLayer>(app: &mut App) {
    app.add_systems(
        PreUpdate,
        (update_key_fields, update_text_fields).after(InputSystem),
    )
    .add_systems(
        PostUpdate,
        (
            update_key_field_focus,
            (
                update_text_field_focus,
                caret_blink,
                layout::<L>.before(PxSet::Picking),
            )
                .chain(),
        ),
    );
}

pub trait PxUiBuilder<M>: 'static {
    fn hide(self) -> impl PxUiBuilder<()>
    where
        Self: Sized,
    {
        move |mut entity: EntityCommands| {
            entity.insert(Visibility::Hidden);
            self.insert_into(entity);
        }
    }

    fn observe<E: Event, B: Bundle, M2>(
        self,
        observer: impl IntoObserverSystem<E, B, M2>,
    ) -> impl PxUiBuilder<()>
    where
        Self: Sized,
    {
        move |mut entity: EntityCommands| {
            self.insert_into(entity.reborrow());
            entity.observe(observer);
        }
    }

    fn spawn<'a>(self, cmd: &'a mut Commands) -> EntityCommands<'a>
    where
        Self: Sized,
    {
        Box::new(self).dyn_spawn(cmd)
    }

    fn spawn_root<'a, L: PxLayer>(self, layer: L, cmd: &'a mut Commands) -> EntityCommands<'a>
    where
        Self: Sized,
    {
        let mut root = self.spawn(cmd);
        root.insert((layer, PxCanvas::Camera));
        root
    }

    fn dyn_spawn<'a>(self: Box<Self>, cmd: &'a mut Commands) -> EntityCommands<'a> {
        let mut entity = cmd.spawn_empty();
        self.dyn_insert_into(entity.reborrow());
        entity
    }

    fn insert_into(self, entity: EntityCommands)
    where
        Self: Sized,
    {
        Box::new(self).dyn_insert_into(entity)
    }

    fn dyn_insert_into(self: Box<Self>, entity: EntityCommands);

    fn erase(self) -> impl PxUiBuilder<()>
    where
        Self: Sized;
}

impl PxUiBuilder<()> for Entity {
    fn dyn_spawn<'a>(self: Box<Self>, cmd: &'a mut Commands) -> EntityCommands<'a> {
        cmd.entity(*self)
    }

    fn dyn_insert_into(self: Box<Self>, _: EntityCommands) {
        error!("Called `dyn_insert_into` on `Entity`")
    }

    fn erase(self) -> impl PxUiBuilder<()>
    where
        Self: Sized,
    {
        self
    }
}

impl<T: 'static + FnOnce(EntityCommands)> PxUiBuilder<()> for T {
    fn dyn_insert_into(self: Box<Self>, entity: EntityCommands) {
        self(entity);
    }

    fn erase(self) -> impl PxUiBuilder<()> {
        self
    }
}

impl<M, U: PxUiBuilder<M>, M2, T: 'static + IntoSystem<(), U, M2> + Send> PxUiBuilder<(M, U, M2)>
    for T
{
    fn dyn_insert_into(self: Box<Self>, mut entity: EntityCommands) {
        entity.queue(|id: Entity, world: &mut World| {
            let mut system = IntoSystem::into_system(*self);
            system.initialize(world);
            let ui = system.run((), world);
            let mut cmd = world.commands();
            let Some(entity) = cmd.get_entity(id) else {
                return;
            };
            ui.insert_into(entity);
        });
    }

    fn erase(self) -> impl PxUiBuilder<()>
    where
        Self: Sized,
    {
        move |entity: EntityCommands| self.insert_into(entity)
    }
}

pub trait PxSlot {
    fn new(content: Entity) -> Self;
}

pub trait PxSlotBuilder<T: PxSlot, M>: 'static {
    fn spawn<'a>(self: Box<Self>, cmd: &'a mut Commands) -> (T, EntityCommands<'a>);

    fn erase(self) -> impl PxSlotBuilder<T, ()>
    where
        Self: Sized;
}

impl<M, T: PxUiBuilder<M>, U: PxSlot> PxSlotBuilder<U, M> for T {
    fn spawn<'a>(self: Box<Self>, cmd: &'a mut Commands) -> (U, EntityCommands<'a>) {
        let content = (*self).spawn(cmd);
        (U::new(content.id()), content)
    }

    fn erase(self) -> impl PxSlotBuilder<U, ()>
    where
        Self: Sized,
    {
        self.erase()
    }
}

#[derive(Component)]
pub struct PxSpace;

impl PxUiBuilder<()> for PxSpace {
    fn dyn_insert_into(self: Box<Self>, mut entity: EntityCommands) {
        entity.insert(*self);
    }

    fn erase(self) -> impl PxUiBuilder<()>
    where
        Self: Sized,
    {
        self
    }
}

#[derive(Component, Reflect)]
#[require(Visibility)]
pub struct PxContainer {
    content: Entity,
    margin: u32,
}

impl PxContainer {
    pub fn build<M>(content: impl PxUiBuilder<M>) -> PxContainerBuilder {
        PxContainerBuilder {
            content: Box::new(content.erase()),
            margin: 0,
        }
    }
}

pub struct PxContainerBuilder {
    pub content: Box<dyn PxUiBuilder<()>>,
    pub margin: u32,
}

impl PxContainerBuilder {
    pub fn margin(mut self, margin: u32) -> Self {
        self.margin = margin;
        self
    }
}

impl PxUiBuilder<()> for PxContainerBuilder {
    fn dyn_insert_into(self: Box<Self>, mut entity: EntityCommands) {
        let content_id = self.content.dyn_spawn(entity.commands_mut()).id();

        entity
            .try_insert(PxContainer {
                content: content_id,
                margin: self.margin,
            })
            .add_child(content_id);
    }

    fn erase(self) -> impl PxUiBuilder<()>
    where
        Self: Sized,
    {
        self
    }
}

#[derive(Clone, Reflect)]
pub struct PxRowSlot {
    pub content: Entity,
    pub stretch: bool,
    // TODO Maybe make filling the default?
    pub fill: bool,
}

impl PxRowSlot {
    pub fn build<M>(content: impl PxUiBuilder<M>) -> PxRowSlotBuilder {
        PxRowSlotBuilder {
            content: Box::new(content.erase()),
            stretch: false,
            fill: false,
        }
    }
}

impl PxSlot for PxRowSlot {
    fn new(content: Entity) -> Self {
        Self {
            content,
            stretch: false,
            fill: false,
        }
    }
}

pub struct PxRowSlotBuilder {
    pub content: Box<dyn PxUiBuilder<()>>,
    pub stretch: bool,
    pub fill: bool,
}

impl PxRowSlotBuilder {
    pub fn stretch(mut self) -> Self {
        self.stretch = true;
        self
    }

    pub fn fill(mut self) -> Self {
        self.fill = true;
        self
    }
}

impl PxSlotBuilder<PxRowSlot, ()> for PxRowSlotBuilder {
    fn spawn<'a>(self: Box<Self>, cmd: &'a mut Commands) -> (PxRowSlot, EntityCommands<'a>) {
        let content = self.content.dyn_spawn(cmd);
        (
            PxRowSlot {
                content: content.id(),
                stretch: self.stretch,
                fill: self.fill,
            },
            content,
        )
    }

    fn erase(self) -> impl PxSlotBuilder<PxRowSlot, ()>
    where
        Self: Sized,
    {
        self
    }
}

#[derive(Component, Clone, Reflect)]
#[require(Visibility)]
pub struct PxRow {
    pub entries: Vec<PxRowSlot>,
    pub vertical: bool,
    pub space_between: u32,
    pub scroll: Option<u32>,
}

impl PxRow {
    pub fn build() -> PxRowBuilder {
        PxRowBuilder {
            entries: Vec::new(),
            vertical: false,
            space_between: 0,
            scroll: false,
        }
    }
}

impl PxRow {
    pub fn push_entry<M>(
        &mut self,
        entry: impl PxSlotBuilder<PxRowSlot, M>,
        mut row: EntityCommands,
    ) {
        let (entry, entry_entity) = Box::new(entry).spawn(row.commands_mut());
        let entry_id = entry_entity.id();

        self.entries.push(entry);
        row.add_child(entry_id);
    }

    pub fn clear_entries(&mut self, mut row: EntityCommands) {
        self.entries.clear();
        row.try_despawn_descendants();
    }
}

pub struct PxRowBuilder {
    pub entries: Vec<Box<dyn PxSlotBuilder<PxRowSlot, ()>>>,
    pub vertical: bool,
    pub space_between: u32,
    pub scroll: bool,
}

impl PxRowBuilder {
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }

    pub fn entry<M>(mut self, entry: impl PxSlotBuilder<PxRowSlot, M>) -> Self {
        self.entries.push(Box::new(entry.erase()));
        self
    }

    pub fn space_between(mut self, space_between: u32) -> Self {
        self.space_between = space_between;
        self
    }

    pub fn scroll(mut self) -> Self {
        self.scroll = true;
        self
    }
}

impl PxUiBuilder<()> for PxRowBuilder {
    fn dyn_insert_into(self: Box<Self>, mut entity: EntityCommands) {
        let entries = self
            .entries
            .into_iter()
            .map(|entry| {
                let (entry, entry_entity) = entry.spawn(entity.commands_mut());
                let entry_id = entry_entity.id();
                entity.add_child(entry_id);
                entry
            })
            .collect();

        entity.try_insert(PxRow {
            entries,
            vertical: self.vertical,
            space_between: self.space_between,
            scroll: self.scroll.then_some(0),
        });
    }

    fn erase(self) -> impl PxUiBuilder<()>
    where
        Self: Sized,
    {
        self
    }
}

#[derive(Default, Clone)]
pub struct GridRow {
    pub stretch: bool,
}

#[derive(Clone)]
pub struct GridRows {
    pub rows: Vec<GridRow>,
    pub space_between: u32,
}

#[derive(Component, Clone)]
#[require(Visibility)]
pub struct PxGrid {
    pub width: u32,
    pub entries: Vec<Entity>,
    pub rows: GridRows,
    pub columns: GridRows,
}

impl PxGrid {
    pub fn build(width: u32) -> PxGridBuilder {
        PxGridBuilder {
            width,
            entries: Vec::new(),
            rows: GridRows {
                rows: Vec::new(),
                space_between: 0,
            },
            columns: GridRows {
                rows: vec![default(); width as usize],
                space_between: 0,
            },
        }
    }
}

pub struct PxGridBuilder {
    pub width: u32,
    pub entries: Vec<Box<dyn PxUiBuilder<()>>>,
    pub rows: GridRows,
    pub columns: GridRows,
}

impl PxGridBuilder {
    pub fn entry<M>(mut self, entry: impl PxUiBuilder<M>) -> Self {
        self.entries.push(Box::new(entry.erase()));
        self
    }

    pub fn rows_space_between(mut self, space_between: u32) -> Self {
        self.rows.space_between = space_between;
        self
    }

    pub fn columns_space_between(mut self, space_between: u32) -> Self {
        self.columns.space_between = space_between;
        self
    }

    pub fn row_stretch(mut self, row: usize) -> Self {
        if self.rows.rows.len() < row {
            self.rows.rows.resize(row + 1, default());
        }

        self.rows.rows[row].stretch = true;
        self
    }

    pub fn column_stretch(mut self, column: usize) -> Self {
        if let Some(column) = self.columns.rows.get_mut(column) {
            column.stretch = true;
        }

        self
    }
}

impl PxUiBuilder<()> for PxGridBuilder {
    fn dyn_insert_into(self: Box<Self>, mut entity: EntityCommands) {
        let entries = self
            .entries
            .into_iter()
            .map(|entry| {
                let entry = entry.dyn_spawn(entity.commands_mut()).id();
                entity.add_child(entry);
                entry
            })
            .collect();

        entity.try_insert(PxGrid {
            width: self.width,
            entries,
            rows: self.rows,
            columns: self.columns,
        });
    }

    fn erase(self) -> impl PxUiBuilder<()>
    where
        Self: Sized,
    {
        self
    }
}

#[derive(Component, Clone)]
#[require(Visibility)]
pub struct PxStack {
    pub entries: Vec<Entity>,
}

impl PxStack {
    pub fn build() -> PxStackBuilder {
        PxStackBuilder {
            entries: Vec::new(),
        }
    }
}

pub struct PxStackBuilder {
    pub entries: Vec<Box<dyn PxUiBuilder<()>>>,
}

impl PxStackBuilder {
    pub fn entry<M>(mut self, entry: impl PxUiBuilder<M>) -> Self {
        self.entries.push(Box::new(entry.erase()));
        self
    }
}

impl PxUiBuilder<()> for PxStackBuilder {
    fn dyn_insert_into(self: Box<Self>, mut entity: EntityCommands) {
        let entries = self
            .entries
            .into_iter()
            .map(|entry| {
                let entry = entry.dyn_spawn(entity.commands_mut()).id();
                entity.add_child(entry);
                entry
            })
            .collect();

        entity.try_insert(PxStack { entries });
    }

    fn erase(self) -> impl PxUiBuilder<()>
    where
        Self: Sized,
    {
        self
    }
}

impl PxRect {
    pub fn build(filter: Handle<PxFilterAsset>) -> PxRectBuilder {
        PxRectBuilder { filter }
    }
}

pub struct PxRectBuilder {
    filter: Handle<PxFilterAsset>,
}

impl PxUiBuilder<()> for PxRectBuilder {
    fn dyn_insert_into(self: Box<Self>, mut entity: EntityCommands) {
        entity.try_insert((
            PxRect(UVec2::ZERO),
            PxFilter(self.filter),
            DefaultPxFilterLayers { clip: false },
        ));
    }

    fn erase(self) -> impl PxUiBuilder<()>
    where
        Self: Sized,
    {
        self
    }
}

impl PxSprite {
    pub fn build(sprite: Handle<PxSpriteAsset>) -> PxSpriteBuilder {
        PxSpriteBuilder {
            sprite,
            filter: None,
            animation: None,
        }
    }
}

pub struct PxSpriteBuilder {
    pub sprite: Handle<PxSpriteAsset>,
    pub filter: Option<Handle<PxFilterAsset>>,
    pub animation: Option<PxAnimation>,
}

impl PxSpriteBuilder {
    pub fn filter(mut self, filter: Handle<PxFilterAsset>) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn animation(mut self, animation: PxAnimation) -> Self {
        self.animation = Some(animation);
        self
    }
}

impl PxUiBuilder<()> for PxSpriteBuilder {
    fn dyn_insert_into(self: Box<Self>, mut entity: EntityCommands) {
        entity.try_insert(PxSprite(self.sprite));

        if let Some(filter) = self.filter {
            entity.try_insert(PxFilter(filter));
        }

        if let Some(animation) = self.animation {
            entity.try_insert(animation);
        }
    }

    fn erase(self) -> impl PxUiBuilder<()>
    where
        Self: Sized,
    {
        self
    }
}

impl PxText {
    pub fn build(value: impl Into<String>, typeface: Handle<PxTypeface>) -> PxTextBuilder {
        PxTextBuilder {
            value: value.into(),
            typeface,
            filter: None,
            animation: None,
        }
    }
}

pub struct PxTextBuilder {
    pub value: String,
    pub typeface: Handle<PxTypeface>,
    pub filter: Option<Handle<PxFilterAsset>>,
    pub animation: Option<PxAnimation>,
}

impl PxTextBuilder {
    pub fn filter(mut self, filter: Handle<PxFilterAsset>) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn animation(mut self, animation: PxAnimation) -> Self {
        self.animation = Some(animation);
        self
    }
}

impl PxUiBuilder<()> for PxTextBuilder {
    fn dyn_insert_into(self: Box<Self>, mut entity: EntityCommands) {
        entity.try_insert(PxText::new(self.value, self.typeface));

        if let Some(filter) = self.filter {
            entity.try_insert(PxFilter(filter));
        }

        if let Some(animation) = self.animation {
            entity.try_insert(animation);
        }
    }

    fn erase(self) -> impl PxUiBuilder<()>
    where
        Self: Sized,
    {
        self
    }
}

#[derive(Component)]
#[require(PxText)]
pub struct PxKeyField {
    pub caret: char,
    /// System that creates the text label
    ///
    /// Ideally, this would accept a Bevy `Key`, but there doesn't seem to be a way to convert a
    /// winit `PhysicalKey` to a winit `Key`, so it wouldn't be possible to run this when building
    /// the UI (ie in `PxUiBuilder::dyn_insert_into`) or update all the text if the keyboard layout
    /// changes.
    pub key_to_str: SystemId<In<KeyCode>, String>,
    pub cached_text: String,
}

impl PxKeyField {
    pub fn build(
        key: KeyCode,
        caret: char,
        key_to_str: SystemId<In<KeyCode>, String>,
        typeface: Handle<PxTypeface>,
    ) -> PxKeyFieldBuilder {
        PxKeyFieldBuilder {
            key,
            caret,
            key_to_str,
            typeface,
        }
    }
}

pub struct PxKeyFieldBuilder {
    pub key: KeyCode,
    pub caret: char,
    pub typeface: Handle<PxTypeface>,
    pub key_to_str: SystemId<In<KeyCode>, String>,
}

impl PxUiBuilder<()> for PxKeyFieldBuilder {
    fn dyn_insert_into(self: Box<Self>, mut entity: EntityCommands) {
        entity.queue(|id: Entity, world: &mut World| {
            let Ok(text) = world.run_system_with_input(self.key_to_str, self.key) else {
                error!("couldn't run `key_to_str`");
                return;
            };

            let Ok(mut entity) = world.get_entity_mut(id) else {
                return;
            };

            entity.insert((
                PxKeyField {
                    caret: self.caret,
                    key_to_str: self.key_to_str,
                    cached_text: text.clone(),
                },
                PxText::new(text, self.typeface),
            ));
        });
    }

    fn erase(self) -> impl PxUiBuilder<()>
    where
        Self: Sized,
    {
        self
    }
}

fn update_key_field_focus(
    mut prev_focus: Local<Option<Entity>>,
    mut fields: Query<(&PxKeyField, &mut PxText, &mut Visibility, Entity)>,
    focus: Res<Focus>,
    mut cmd: Commands,
) {
    if *prev_focus == **focus {
        return;
    }

    if let Some(prev_focus) = *prev_focus {
        if let Ok((field, mut text, mut visibility, id)) = fields.get_mut(prev_focus) {
            text.value = field.cached_text.clone();
            *visibility = Visibility::Inherited;
            cmd.entity(id).remove::<Blink>();
        }
    }

    if let Some(focus) = **focus {
        if let Ok((field, mut text, _, id)) = fields.get_mut(focus) {
            text.value = field.caret.to_string();
            cmd.entity(id)
                .try_insert(Blink::new(Duration::from_millis(500)));
        }
    }

    *prev_focus = **focus;
}

#[derive(Event)]
pub struct PxKeyFieldUpdate {
    pub key: KeyCode,
}

fn update_key_fields(
    mut fields: Query<Entity, With<PxKeyField>>,
    mut focus: ResMut<Focus>,
    mut keys: EventReader<KeyboardInput>,
    mut cmd: Commands,
) {
    let mut keys = keys.read();
    let key = keys.find(|key| matches!(key.state, ButtonState::Pressed));
    keys.last();
    let Some(key) = key else {
        return;
    };

    let Some(focus_id) = **focus else {
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

        let key = match world.run_system_with_input(field.key_to_str, key) {
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

    cmd.entity(field_id).trigger(PxKeyFieldUpdate { key });

    **focus = None;
}

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

#[derive(Component)]
#[require(PxText)]
pub struct PxTextField {
    pub cached_text: String,
    pub caret_char: char,
    pub caret: Option<PxCaret>,
}

impl PxTextField {
    pub fn build(caret: char, typeface: Handle<PxTypeface>) -> PxTextFieldBuilder {
        PxTextFieldBuilder { caret, typeface }
    }
}

pub struct PxTextFieldBuilder {
    pub caret: char,
    pub typeface: Handle<PxTypeface>,
}

impl PxUiBuilder<()> for PxTextFieldBuilder {
    fn dyn_insert_into(self: Box<Self>, mut entity: EntityCommands) {
        entity.try_insert((
            PxTextField {
                cached_text: String::new(),
                caret_char: self.caret,
                caret: None,
            },
            PxText::new(String::new(), self.typeface),
        ));
    }

    fn erase(self) -> impl PxUiBuilder<()>
    where
        Self: Sized,
    {
        self
    }
}

fn update_text_field_focus(
    mut prev_focus: Local<Option<Entity>>,
    mut fields: Query<(&mut PxTextField, &mut PxText)>,
    focus: Res<Focus>,
) {
    if *prev_focus == **focus {
        return;
    }

    if let Some(prev_focus) = *prev_focus {
        if let Ok((mut field, mut text)) = fields.get_mut(prev_focus) {
            text.value = field.cached_text.clone();
            field.caret = None;
        }
    }

    if let Some(focus) = **focus {
        if let Ok((mut field, mut text)) = fields.get_mut(focus) {
            field.cached_text = text.value.clone();
            text.value += &field.caret_char.to_string();
            field.caret = Some(default());
        }
    }

    *prev_focus = **focus;
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

#[derive(Event)]
pub struct PxTextFieldUpdate {
    pub text: String,
}

fn update_text_fields(
    mut fields: Query<(&mut PxTextField, &mut PxText)>,
    focus: Res<Focus>,
    mut keys: EventReader<KeyboardInput>,
    mut cmd: Commands,
) {
    let keys = keys
        .read()
        .filter(|key| matches!(key.state, ButtonState::Pressed))
        .collect::<Vec<_>>();

    if keys.is_empty() {
        return;
    }

    let Some(focus_id) = **focus else {
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

    cmd.entity(focus_id).trigger(PxTextFieldUpdate {
        text: field.cached_text.clone(),
    });
}

// If layouting ends up being too slow, make a tree of min sizes up front and lookup in that
fn min_size<L: PxLayer>(
    ui: Entity,
    uis: Query<(
        AnyOf<(
            &PxContainer,
            &PxRow,
            &PxGrid,
            &PxStack,
            (&PxRect, &PxFilterLayers<L>),
            &PxSprite,
            &PxText,
        )>,
        Option<&L>,
        Option<(&PxPosition, &PxCanvas)>,
    )>,
    typefaces: &Assets<PxTypeface>,
    sprites: &Assets<PxSpriteAsset>,
) -> UVec2 {
    let Ok(((container, row, grid, stack, rect, sprite, text), _, _)) = uis.get(ui) else {
        // This includes `PxSpace`. Surprise, the `PxSpace` component doesn't do anything at all.
        // It's just easier to spawn in UI.
        return UVec2::ZERO;
    };

    if let Some(container) = container {
        return min_size(container.content, uis.to_readonly(), typefaces, sprites)
            + 2 * UVec2::splat(container.margin);
    }

    if let Some(row) = row {
        fn main(vec: UVec2, vert: bool) -> u32 {
            if vert {
                vec.y
            } else {
                vec.x
            }
        }

        fn main_mut(vec: &mut UVec2, vert: bool) -> &mut u32 {
            if vert {
                &mut vec.y
            } else {
                &mut vec.x
            }
        }

        fn cross(vec: UVec2, vert: bool) -> u32 {
            if vert {
                vec.x
            } else {
                vec.y
            }
        }

        fn cross_mut(vec: &mut UVec2, vert: bool) -> &mut u32 {
            if vert {
                &mut vec.x
            } else {
                &mut vec.y
            }
        }

        let vert = row.vertical;
        let mut size = UVec2::ZERO;

        *main_mut(&mut size, vert) +=
            row.entries.len().saturating_sub(1) as u32 * row.space_between;

        for entry in &row.entries {
            let min_size = min_size(entry.content, uis.to_readonly(), typefaces, sprites);

            *main_mut(&mut size, vert) += main(min_size, vert);

            let cross_size = cross(min_size, vert);
            if cross_size > cross(size, vert) {
                *cross_mut(&mut size, vert) = cross_size;
            }
        }

        return size;
    }

    if let Some(grid) = grid {
        let mut column_widths = vec![0; grid.width as usize];
        let mut height = (grid.entries.len() as u32)
            .div_ceil(grid.width)
            .saturating_sub(1)
            * grid.rows.space_between;

        for row in grid.entries.chunks(grid.width as usize) {
            let mut row_height = 0;

            for (column, &entry) in row.iter().enumerate() {
                let size = min_size(entry, uis.to_readonly(), typefaces, sprites);

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

    if let Some(stack) = stack {
        let mut size = UVec2::ZERO;

        for &entry in &stack.entries {
            size = size.max(min_size(entry, uis.to_readonly(), typefaces, sprites));
        }

        return size;
    }

    if rect.is_some() {
        return UVec2::ZERO;
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
            &PxContainer,
            &PxRow,
            &PxGrid,
            &PxStack,
            (&mut PxRect, &mut PxFilterLayers<L>),
            &PxSprite,
            &mut PxText,
        )>,
        Option<&mut L>,
        Option<(&mut PxPosition, &mut PxCanvas)>,
    )>,
    typefaces: &Assets<PxTypeface>,
    sprites: &Assets<PxSpriteAsset>,
) {
    let Ok(((container, row, grid, stack, rect, sprite, text), _, _)) = uis.get(ui) else {
        return;
    };

    if let Some(container) = container {
        layout_inner(
            IRect {
                min: target_rect.min + container.margin as i32,
                max: target_rect.max - container.margin as i32,
            },
            target_layer,
            target_canvas,
            container.content,
            uis,
            typefaces,
            sprites,
        );

        return;
    }

    // TODO Scrolling
    if let Some(row) = row.cloned() {
        // TODO Dedup these functions, just use !vert for cross
        fn main(vec: IVec2, vert: bool) -> i32 {
            if vert {
                vec.y
            } else {
                vec.x
            }
        }

        fn main_mut(vec: &mut IVec2, vert: bool) -> &mut i32 {
            if vert {
                &mut vec.y
            } else {
                &mut vec.x
            }
        }

        // Adds to x, subtracts from y
        fn main_add(vec: &mut IVec2, addend: i32, vert: bool) {
            if vert {
                vec.y -= addend;
            } else {
                vec.x += addend;
            }
        }

        fn cross_mut(vec: &mut IVec2, vert: bool) -> &mut i32 {
            if vert {
                &mut vec.x
            } else {
                &mut vec.y
            }
        }

        fn rect_size_main(rect: IRect, vert: bool) -> i32 {
            if vert {
                rect.height()
            } else {
                rect.width()
            }
        }

        fn rect_size_cross(rect: IRect, vert: bool) -> i32 {
            if vert {
                rect.width()
            } else {
                rect.height()
            }
        }

        let vert = row.vertical;
        let mut pos = ivec2(target_rect.min.x, target_rect.max.y);
        let mut remaining_stretchers =
            row.entries.iter().filter(|entry| entry.stretch).count() as i32;
        let mut stretch_budget = rect_size_main(target_rect, vert)
            - main(
                min_size(ui, uis.to_readonly(), typefaces, sprites).as_ivec2(),
                vert,
            );
        let fill_size = rect_size_cross(target_rect, vert);

        for entry in row.entries {
            let mut size =
                min_size(entry.content, uis.to_readonly(), typefaces, sprites).as_ivec2();
            if entry.stretch {
                // For simplicity, we just split the extra size among the stretched entries evenly
                // instead of prioritizing the smallest. I might change this in the future.
                let extra_size = stretch_budget / remaining_stretchers;
                *main_mut(&mut size, vert) += extra_size;
                stretch_budget -= extra_size;
                remaining_stretchers -= 1;
            }

            if entry.fill {
                *cross_mut(&mut size, vert) = fill_size;
            }

            layout_inner(
                IRect {
                    min: ivec2(pos.x, pos.y - size.y),
                    max: ivec2(pos.x + size.x, pos.y),
                },
                target_layer,
                target_canvas,
                entry.content,
                uis.reborrow(),
                typefaces,
                sprites,
            );

            main_add(&mut pos, main(size, vert) + row.space_between as i32, vert);
        }

        return;
    }

    if let Some(grid) = grid.cloned() {
        let mut column_widths = vec![0; grid.width as usize];
        let mut row_heights = vec![0; grid.entries.len().div_ceil(grid.width as usize)];

        for (row_index, row) in grid.entries.chunks(grid.width as usize).enumerate() {
            for (column, &entry) in row.iter().enumerate() {
                let size = min_size(entry, uis.to_readonly(), typefaces, sprites).as_ivec2();

                if size.x > column_widths[column] {
                    column_widths[column] = size.x;
                }

                if size.y > row_heights[row_index] {
                    row_heights[row_index] = size.y;
                }
            }
        }

        let min_size = min_size(ui, uis.to_readonly(), typefaces, sprites).as_ivec2();

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

        for (row_index, row) in grid.entries.chunks(grid.width as usize).enumerate() {
            let mut x_pos = target_rect.min.x;
            let height = row_heights[row_index];

            for (column, &entry) in row.iter().enumerate() {
                let width = column_widths[column];

                layout_inner(
                    IRect {
                        min: ivec2(x_pos, y_pos - height),
                        max: ivec2(x_pos + width, y_pos),
                    },
                    target_layer,
                    target_canvas,
                    entry,
                    uis.reborrow(),
                    typefaces,
                    sprites,
                );

                x_pos += width + grid.columns.space_between as i32;
            }

            y_pos -= height + grid.columns.space_between as i32;
        }

        return;
    }

    if let Some(stack) = stack.cloned() {
        let mut layer = target_layer.clone();

        for entry in stack.entries {
            layout_inner(
                target_rect,
                &layer,
                target_canvas,
                entry,
                uis.reborrow(),
                typefaces,
                sprites,
            );

            if let Some(next) = layer.clone().next() {
                layer = next;
            }
        }

        return;
    }

    if rect.is_some() {
        let ((_, _, _, _, rect, _, _), _, pos) = uis.get_mut(ui).unwrap();

        if let Some((mut pos, mut canvas)) = pos {
            **pos = target_rect.center();
            *canvas = target_canvas;
        }

        let (mut rect, mut layers) = rect.unwrap();

        let target_layer = target_layer.clone();
        match *layers {
            ref mut layers @ (PxFilterLayers::Many(_) | PxFilterLayers::Select(_)) => {
                *layers = PxFilterLayers::single_over(target_layer)
            }
            PxFilterLayers::Single { ref mut layer, .. } => *layer = target_layer,
        }

        **rect = target_rect.size().as_uvec2();

        return;
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

        return;
    }

    if text.is_some() {
        let ((_, _, _, _, _, _, text), layer, pos) = uis.get_mut(ui).unwrap();

        if let Some(mut layer) = layer {
            *layer = target_layer.clone();
        }

        let Some((mut pos, mut canvas)) = pos else {
            return;
        };

        *canvas = target_canvas;

        let mut text = text.unwrap();
        let PxText {
            ref mut value,
            ref typeface,
            ref mut line_breaks,
        } = *text;

        let Some(typeface) = typefaces.get(typeface) else {
            return;
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

        return;
    }

    unreachable!();
}

fn layout<L: PxLayer>(
    mut uis: ParamSet<(
        Query<
            (&L, &PxCanvas, Entity),
            (
                Or<(With<PxContainer>, With<PxRow>, With<PxGrid>, With<PxStack>)>,
                Without<Parent>,
            ),
        >,
        Query<(
            AnyOf<(
                &PxContainer,
                &PxRow,
                &PxGrid,
                &PxStack,
                (&mut PxRect, &mut PxFilterLayers<L>),
                &PxSprite,
                &mut PxText,
            )>,
            Option<&mut L>,
            Option<(&mut PxPosition, &mut PxCanvas)>,
        )>,
    )>,
    typefaces: Res<Assets<PxTypeface>>,
    sprites: Res<Assets<PxSpriteAsset>>,
    screen: Res<Screen>,
) {
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
            &typefaces,
            &sprites,
        );
    }
}

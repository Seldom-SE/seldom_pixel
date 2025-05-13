//! `seldom_pixel`'s UI system. The building blocks of UI are here, but they are all just pieces.
//! For example, there is a [`PxTextField`] component, but if you spawn it on its own, the text
//! field won't have a background, and you won't even be able to type in it. Instead, you should
//! make your own helper functions that compose UI builders together. For a text field, you could
//! use a [`PxStack`] with a white [`PxRect`] background and a [`PxTextField`], and add an observer
//! on [`PxRect`] that sets [`Focus`] to the text field.
//!
//! For more information, browse this module and see the `ui` example.

// TODO UI example
// TODO Feature parity between widgets
// TODO Split into modules

use std::time::Duration;

use bevy::{
    a11y::Focus,
    ecs::system::{IntoObserverSystem, SystemId},
    input::{
        keyboard::{Key, KeyboardInput, NativeKey},
        mouse::MouseWheel,
        ButtonState, InputSystem,
    },
    math::{ivec2, uvec2},
};

use crate::{
    blink::Blink, filter::TRANSPARENT_FILTER, position::Spatial, prelude::*, screen::Screen,
    set::PxSet,
};

pub(crate) fn plug<L: PxLayer>(app: &mut App) {
    app.add_systems(
        PreUpdate,
        (update_key_fields, update_text_fields, scroll).after(InputSystem),
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
        self.insert(Visibility::Hidden)
    }

    fn insert(self, bundle: impl Bundle) -> impl PxUiBuilder<()>
    where
        Self: Sized,
    {
        move |mut entity: EntityCommands| {
            entity.insert(bundle);
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

#[derive(Component)]
#[require(Visibility)]
pub struct PxMinSize {
    pub content: Entity,
    pub x: u32,
    pub y: u32,
}

impl PxMinSize {
    pub fn build<M>(content: impl PxUiBuilder<M>) -> PxMinSizeBuilder {
        PxMinSizeBuilder {
            content: Box::new(content.erase()),
            x: 0,
            y: 0,
        }
    }
}

pub struct PxMinSizeBuilder {
    pub content: Box<dyn PxUiBuilder<()>>,
    pub x: u32,
    pub y: u32,
}

impl PxMinSizeBuilder {
    pub fn x(mut self, x: u32) -> Self {
        self.x = x;
        self
    }

    pub fn y(mut self, y: u32) -> Self {
        self.y = y;
        self
    }
}

impl PxUiBuilder<()> for PxMinSizeBuilder {
    fn dyn_insert_into(self: Box<Self>, mut entity: EntityCommands) {
        let content = self.content.dyn_spawn(entity.commands_mut()).id();

        entity
            .try_insert(PxMinSize {
                content,
                x: self.x,
                y: self.y,
            })
            .add_child(content);
    }

    fn erase(self) -> impl PxUiBuilder<()> {
        self
    }
}

// TODO Rename to `PxMargin`
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
        let content = self.content.dyn_spawn(entity.commands_mut()).id();

        entity
            .try_insert(PxContainer {
                content,
                margin: self.margin,
            })
            .add_child(content);
    }

    fn erase(self) -> impl PxUiBuilder<()> {
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

    fn erase(self) -> impl PxSlotBuilder<PxRowSlot, ()> {
        self
    }
}

#[derive(Component, Clone, Reflect)]
#[require(Visibility)]
pub struct PxRow {
    pub entries: Vec<PxRowSlot>,
    pub vertical: bool,
    pub space_between: u32,
}

impl PxRow {
    pub fn build() -> PxRowBuilder {
        PxRowBuilder {
            entries: Vec::new(),
            vertical: false,
            space_between: 0,
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
        });
    }

    fn erase(self) -> impl PxUiBuilder<()> {
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

    fn erase(self) -> impl PxUiBuilder<()> {
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

    fn erase(self) -> impl PxUiBuilder<()> {
        self
    }
}

#[derive(Component, Clone, Copy)]
#[require(PxInvertMask, PxRect)]
pub struct PxScroll {
    pub content: Entity,
    pub bar: Entity,
    pub bar_bg: Entity,
    pub horizontal: bool,
    pub scroll: u32,
    pub max_scroll: u32,
}

impl PxScroll {
    pub fn build<M1, M2, M3>(
        content: impl PxUiBuilder<M1>,
        bar: impl PxUiBuilder<M2>,
        bar_bg: impl PxUiBuilder<M3>,
    ) -> PxScrollBuilder {
        PxScrollBuilder {
            content: Box::new(content.erase()),
            bar: Box::new(bar.erase()),
            bar_bg: Box::new(bar_bg.erase()),
            horizontal: false,
        }
    }
}

pub struct PxScrollBuilder {
    pub content: Box<dyn PxUiBuilder<()>>,
    pub bar: Box<dyn PxUiBuilder<()>>,
    pub bar_bg: Box<dyn PxUiBuilder<()>>,
    pub horizontal: bool,
}

impl PxScrollBuilder {
    pub fn horizontal(mut self) -> Self {
        self.horizontal = true;
        self
    }
}

impl PxUiBuilder<()> for PxScrollBuilder {
    fn dyn_insert_into(self: Box<Self>, mut entity: EntityCommands) {
        let content = self.content.dyn_spawn(entity.commands_mut()).id();
        let bar = self.bar.dyn_spawn(entity.commands_mut()).id();
        let bar_bg = self.bar_bg.dyn_spawn(entity.commands_mut()).id();

        entity.insert((
            PxScroll {
                content,
                bar,
                bar_bg,
                horizontal: self.horizontal,
                scroll: 0,
                max_scroll: 0,
            },
            PxInvertMask,
            PxFilter(TRANSPARENT_FILTER),
        ));
    }

    fn erase(self) -> impl PxUiBuilder<()> {
        self
    }
}

// TODO Should be modular
fn scroll(mut scrolls: Query<&mut PxScroll>, mut wheels: EventReader<MouseWheel>) {
    for wheel in wheels.read() {
        for mut scroll in &mut scrolls {
            scroll.scroll = scroll
                .scroll
                .saturating_add_signed(-wheel.y as i32)
                .min(scroll.max_scroll);
        }
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
        entity.try_insert((PxRect(UVec2::ZERO), PxFilter(self.filter)));
    }

    fn erase(self) -> impl PxUiBuilder<()> {
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

    fn erase(self) -> impl PxUiBuilder<()> {
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

    fn erase(self) -> impl PxUiBuilder<()> {
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

    fn erase(self) -> impl PxUiBuilder<()> {
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

// TODO Should be modular
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

    fn erase(self) -> impl PxUiBuilder<()> {
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

// TODO Should be modular
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
fn calc_min_size<L: PxLayer>(
    ui: Entity,
    uis: Query<(
        AnyOf<(
            &PxMinSize,
            &PxContainer,
            &PxRow,
            &PxGrid,
            &PxStack,
            (Option<&PxScroll>, &PxRect, &PxFilterLayers<L>),
            &PxSprite,
            &PxText,
        )>,
        Option<&L>,
        Option<(&PxPosition, &PxCanvas)>,
    )>,
    typefaces: &Assets<PxTypeface>,
    sprites: &Assets<PxSpriteAsset>,
) -> UVec2 {
    let Ok(((min_size, container, row, grid, stack, rect, sprite, text), _, _)) = uis.get(ui)
    else {
        // This includes `PxSpace`. Surprise, the `PxSpace` component doesn't do anything at all.
        // It's just easier to spawn in UI.
        return UVec2::ZERO;
    };

    if let Some(min_size) = min_size {
        return calc_min_size(min_size.content, uis.to_readonly(), typefaces, sprites)
            .max(uvec2(min_size.x, min_size.y));
    }

    if let Some(container) = container {
        return calc_min_size(container.content, uis.to_readonly(), typefaces, sprites)
            + 2 * UVec2::splat(container.margin);
    }

    fn dim(vec: UVec2, y: bool) -> u32 {
        if y {
            vec.y
        } else {
            vec.x
        }
    }

    fn dim_mut(vec: &mut UVec2, y: bool) -> &mut u32 {
        if y {
            &mut vec.y
        } else {
            &mut vec.x
        }
    }

    if let Some(row) = row {
        let vert = row.vertical;
        let mut size = UVec2::ZERO;

        *dim_mut(&mut size, vert) += row.entries.len().saturating_sub(1) as u32 * row.space_between;

        for entry in &row.entries {
            let min_size = calc_min_size(entry.content, uis.to_readonly(), typefaces, sprites);

            *dim_mut(&mut size, vert) += dim(min_size, vert);

            let cross_size = dim(min_size, !vert);
            if cross_size > dim(size, !vert) {
                *dim_mut(&mut size, !vert) = cross_size;
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
                let size = calc_min_size(entry, uis.to_readonly(), typefaces, sprites);

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
            size = size.max(calc_min_size(entry, uis.to_readonly(), typefaces, sprites));
        }

        return size;
    }

    if let Some((scroll, _, _)) = rect {
        let Some(scroll) = scroll else {
            return UVec2::ZERO;
        };

        let horz = scroll.horizontal;

        let mut size = calc_min_size(scroll.content, uis.to_readonly(), typefaces, sprites);
        let bar_size = calc_min_size(scroll.bar, uis.to_readonly(), typefaces, sprites).max(
            calc_min_size(scroll.bar_bg, uis.to_readonly(), typefaces, sprites),
        );

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
            &PxMinSize,
            &PxContainer,
            &PxRow,
            &PxGrid,
            &PxStack,
            (Option<&mut PxScroll>, &mut PxRect, &mut PxFilterLayers<L>),
            &PxSprite,
            &mut PxText,
        )>,
        Option<&mut L>,
        Option<(&mut PxPosition, &mut PxCanvas)>,
    )>,
    typefaces: &Assets<PxTypeface>,
    sprites: &Assets<PxSpriteAsset>,
) -> Option<L> {
    let Ok(((min_size, container, row, grid, stack, rect, sprite, text), _, _)) = uis.get(ui)
    else {
        return None;
    };

    if let Some(min_size) = min_size {
        return layout_inner(
            target_rect,
            target_layer,
            target_canvas,
            min_size.content,
            uis,
            typefaces,
            sprites,
        );
    }

    if let Some(container) = container {
        return layout_inner(
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
    }

    fn dim(vec: IVec2, y: bool) -> i32 {
        if y {
            vec.y
        } else {
            vec.x
        }
    }

    // Adds to x, subtracts from y
    fn add(augend: i32, addend: i32, y: bool) -> i32 {
        if y {
            augend - addend
        } else {
            augend + addend
        }
    }

    fn rect_size(rect: IRect, y: bool) -> i32 {
        if y {
            rect.height()
        } else {
            rect.width()
        }
    }

    if let Some(row) = row.cloned() {
        fn dim_mut(vec: &mut IVec2, y: bool) -> &mut i32 {
            if y {
                &mut vec.y
            } else {
                &mut vec.x
            }
        }

        let vert = row.vertical;
        let mut pos = ivec2(target_rect.min.x, target_rect.max.y);
        let mut remaining_stretchers =
            row.entries.iter().filter(|entry| entry.stretch).count() as i32;
        let mut stretch_budget = rect_size(target_rect, vert)
            - dim(
                calc_min_size(ui, uis.to_readonly(), typefaces, sprites).as_ivec2(),
                vert,
            );
        let fill_size = rect_size(target_rect, !vert);

        let mut layer = None::<L>;

        for entry in row.entries {
            let mut size =
                calc_min_size(entry.content, uis.to_readonly(), typefaces, sprites).as_ivec2();
            if entry.stretch {
                // For simplicity, we just split the extra size among the stretched entries evenly
                // instead of prioritizing the smallest. I might change this in the future.
                let extra_size = stretch_budget / remaining_stretchers;
                *dim_mut(&mut size, vert) += extra_size;
                stretch_budget -= extra_size;
                remaining_stretchers -= 1;
            }

            if entry.fill {
                *dim_mut(&mut size, !vert) = fill_size;
            }

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
                entry.content,
                uis.reborrow(),
                typefaces,
                sprites,
            ) {
                layer = Some(last_layer);
            }

            *dim_mut(&mut pos, vert) = add(
                dim(pos, vert),
                dim(size, vert) + row.space_between as i32,
                vert,
            );
        }

        return layer;
    }

    if let Some(grid) = grid.cloned() {
        let mut column_widths = vec![0; grid.width as usize];
        let mut row_heights = vec![0; grid.entries.len().div_ceil(grid.width as usize)];

        for (row_index, row) in grid.entries.chunks(grid.width as usize).enumerate() {
            for (column, &entry) in row.iter().enumerate() {
                let size = calc_min_size(entry, uis.to_readonly(), typefaces, sprites).as_ivec2();

                if size.x > column_widths[column] {
                    column_widths[column] = size.x;
                }

                if size.y > row_heights[row_index] {
                    row_heights[row_index] = size.y;
                }
            }
        }

        let min_size = calc_min_size(ui, uis.to_readonly(), typefaces, sprites).as_ivec2();

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

        for (row_index, row) in grid.entries.chunks(grid.width as usize).enumerate() {
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
                    typefaces,
                    sprites,
                ) {
                    layer = Some(last_layer);
                };

                x_pos += width + grid.columns.space_between as i32;
            }

            y_pos -= height + grid.columns.space_between as i32;
        }

        return layer;
    }

    if let Some(stack) = stack.cloned() {
        let mut layer = None::<L>;

        for entry in stack.entries {
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
                typefaces,
                sprites,
            ) {
                layer = Some(last_layer);
            };
        }

        return layer;
    }

    if rect.is_some() {
        let ((_, _, _, _, _, rect, _, _), _, mut pos) = uis.get_mut(ui).unwrap();

        if let Some((_, ref mut canvas)) = pos {
            **canvas = target_canvas;
        }

        let (scroll, mut rect, mut layers) = rect.unwrap();

        if let Some(scroll) = scroll {
            fn rect_start(rect: IRect, y: bool) -> i32 {
                if y {
                    rect.max.y
                } else {
                    rect.min.x
                }
            }

            fn rect_start_mut(rect: &mut IRect, y: bool) -> &mut i32 {
                if y {
                    &mut rect.max.y
                } else {
                    &mut rect.min.x
                }
            }

            fn rect_end(rect: IRect, y: bool) -> i32 {
                if y {
                    rect.min.y
                } else {
                    rect.max.x
                }
            }

            fn rect_end_mut(rect: &mut IRect, y: bool) -> &mut i32 {
                if y {
                    &mut rect.min.y
                } else {
                    &mut rect.max.x
                }
            }

            let scroll = *scroll;
            let horz = scroll.horizontal;

            let content_min_size =
                calc_min_size(scroll.content, uis.to_readonly(), typefaces, sprites).as_ivec2();

            let bar_min_size = calc_min_size(scroll.bar, uis.to_readonly(), typefaces, sprites)
                .max(calc_min_size(
                    scroll.bar_bg,
                    uis.to_readonly(),
                    typefaces,
                    sprites,
                ))
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
                scroll.content,
                uis.reborrow(),
                typefaces,
                sprites,
            );

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

            let last_bg_layer = layout_inner(
                bar_rect,
                &bg_layer,
                target_canvas,
                scroll.bar_bg,
                uis.reborrow(),
                typefaces,
                sprites,
            );
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
            let mut scroll = scroll.unwrap();

            scroll.max_scroll = (view_size as f32 * (1. / ratio - 1.)).ceil() as u32;

            if let Some(last_bar_layer) = layout_inner(
                bar_rect,
                &bar_layer,
                target_canvas,
                scroll.bar,
                uis.reborrow(),
                typefaces,
                sprites,
            ) {
                layer = Some(last_bar_layer);
            }

            return layer;
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

            return Some(target_layer.clone());
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

        return Some(target_layer.clone());
    }

    if text.is_some() {
        let ((_, _, _, _, _, _, _, text), layer, pos) = uis.get_mut(ui).unwrap();

        if let Some(mut layer) = layer {
            *layer = target_layer.clone();
        }

        let Some((mut pos, mut canvas)) = pos else {
            return Some(target_layer.clone());
        };

        *canvas = target_canvas;

        let mut text = text.unwrap();
        let PxText {
            ref mut value,
            ref typeface,
            ref mut line_breaks,
        } = *text;

        let Some(typeface) = typefaces.get(typeface) else {
            return Some(target_layer.clone());
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

        return Some(target_layer.clone());
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
                &PxMinSize,
                &PxContainer,
                &PxRow,
                &PxGrid,
                &PxStack,
                (Option<&mut PxScroll>, &mut PxRect, &mut PxFilterLayers<L>),
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

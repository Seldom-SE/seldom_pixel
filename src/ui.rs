use bevy::ecs::system::IntoObserverSystem;

use crate::{filter::DefaultPxFilterLayers, prelude::*, screen::Screen};

pub(crate) fn plug(app: &mut App) {
    app.add_systems(PostUpdate, layout);
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
        let mut entity = cmd.spawn_empty();
        self.insert_into(entity.reborrow());
        entity
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
    fn dyn_insert_into(self: Box<Self>, entity: EntityCommands) {}
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
    fn spawn(self: Box<Self>, cmd: Commands) -> T;

    fn erase(self) -> impl PxSlotBuilder<T, ()>
    where
        Self: Sized;
}

impl<M, T: PxUiBuilder<M>, U: PxSlot> PxSlotBuilder<U, M> for T {
    fn spawn(self: Box<Self>, mut cmd: Commands) -> U {
        U::new((*self).spawn(&mut cmd).id())
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
        let content = entity.commands_mut().spawn_empty();
        let content_id = content.id();
        self.content.dyn_insert_into(content);

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

pub struct PxRowSlot {
    pub content: Entity,
    pub stretch: bool,
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
    fn spawn(self: Box<Self>, mut cmd: Commands) -> PxRowSlot {
        PxRowSlot {
            content: self.content.dyn_spawn(&mut cmd).id(),
            stretch: self.stretch,
            fill: self.fill,
        }
    }

    fn erase(self) -> impl PxSlotBuilder<PxRowSlot, ()>
    where
        Self: Sized,
    {
        self
    }
}

#[derive(Component)]
#[require(Visibility)]
pub struct PxRow {
    pub entries: Vec<PxRowSlot>,
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
        let mut commands = entity.commands();

        let entries = self
            .entries
            .into_iter()
            .map(|entry| entry.spawn(commands.reborrow()))
            .collect();

        entity.try_insert(PxRow { entries });
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
        let mut commands = entity.commands();

        let entries = self
            .entries
            .into_iter()
            .map(|entry| entry.dyn_spawn(&mut commands).id())
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

#[derive(Component)]
#[require(DefaultPxFilterLayers, Visibility)]
pub struct PxRect {
    pub rect: IRect,
    pub filter: Handle<PxFilterAsset>,
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
        entity.try_insert(PxRect {
            rect: default(),
            filter: self.filter,
        });
    }

    fn erase(self) -> impl PxUiBuilder<()>
    where
        Self: Sized,
    {
        self
    }
}

impl PxUiBuilder<()> for PxText {
    fn dyn_insert_into(self: Box<Self>, mut entity: EntityCommands) {
        entity.try_insert(*self);
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
pub struct PxTextField;

impl PxTextField {
    pub fn build(typeface: Handle<PxTypeface>) -> PxTextFieldBuilder {
        PxTextFieldBuilder { typeface }
    }
}

pub struct PxTextFieldBuilder {
    pub typeface: Handle<PxTypeface>,
}

impl PxUiBuilder<()> for PxTextFieldBuilder {
    fn dyn_insert_into(self: Box<Self>, mut entity: EntityCommands) {
        entity.try_insert((PxTextField, PxText::new(String::new(), self.typeface)));
    }

    fn erase(self) -> impl PxUiBuilder<()>
    where
        Self: Sized,
    {
        self
    }
}

#[derive(Event)]
pub struct PxTextFieldUpdate {
    pub text: String,
}

fn layout_inner(
    size: UVec2,
    ui: Entity,
    uis: Query<AnyOf<(&PxContainer, &PxStack, (&PxTextField, &PxText))>>,
) {
    let Ok((container, stack, text_field)) = uis.get(ui) else {
        return;
    };

    if let Some(container) = container {
        layout_inner(size, container.content, uis.to_readonly());
    }

    if let Some(stack) = stack {
        for &entry in &stack.entries {
            layout_inner(size, entry, uis.to_readonly());
        }
    }

    if let Some((text_field, text)) = text_field {}
}

fn layout(
    ui_roots: Query<Entity, (With<PxContainer>, Without<Parent>)>,
    uis: Query<AnyOf<(&PxContainer, &PxStack, (&PxTextField, &PxText))>>,
    screen: Res<Screen>,
) {
    for root in &ui_roots {
        layout_inner(screen.computed_size, root, uis.to_readonly());
    }
}

//! Screen and rendering

use std::{collections::BTreeMap, marker::PhantomData};

use bevy::{
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{
            NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{texture_2d, uniform_buffer},
            BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
            ColorTargetState, ColorWrites, DynamicUniformBuffer, Extent3d, FragmentState,
            ImageDataLayout, PipelineCache, RenderPassColorAttachment, RenderPassDescriptor,
            RenderPipelineDescriptor, ShaderStages, ShaderType, TextureDimension, TextureFormat,
            TextureSampleType, TextureViewDescriptor, TextureViewDimension, VertexState,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::{BevyDefault, TextureFormatPixelInfo},
        view::ViewTarget,
        Render, RenderApp, RenderSet,
    },
    window::{PrimaryWindow, WindowResized},
};

#[cfg(feature = "line")]
use crate::line::{draw_line, LineComponents};
use crate::{
    animation::{copy_animation_params, draw_frame, draw_spatial, LastUpdate},
    cursor::{CursorState, PxCursorPosition},
    filter::{draw_filter, FilterComponents},
    frame::{FrameComponents, PxFrame},
    image::{PxImage, PxImageSliceMut},
    map::{MapComponents, PxTile, TileComponents},
    math::RectExt,
    palette::{PaletteHandle, PaletteParam},
    position::{PxLayer, PxSize},
    prelude::*,
    sprite::SpriteComponents,
    text::TextComponents,
};

const SCREEN_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(0x48CE_4F2C_8B78_5954_08A8_461F_62E1_0E84);

pub(crate) struct Plug<L: PxLayer> {
    size: ScreenSize,
    _l: PhantomData<L>,
}

impl<L: PxLayer> Plug<L> {
    pub(crate) fn new(size: ScreenSize) -> Self {
        Self {
            size,
            _l: PhantomData,
        }
    }
}

impl<L: PxLayer> Plugin for Plug<L> {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<Screen>::default())
            .add_systems(Startup, insert_screen(self.size))
            .add_systems(Update, init_screen)
            .add_systems(PostUpdate, (resize_screen, update_screen_palette))
            .world_mut()
            .resource_mut::<Assets<Shader>>()
            .insert(
                SCREEN_SHADER_HANDLE.id(),
                Shader::from_wgsl(include_str!("screen.wgsl"), "screen.wgsl"),
            );

        app.sub_app_mut(RenderApp)
            .add_render_graph_node::<ViewNodeRunner<PxRenderNode<L>>>(Core2d, PxRender)
            .add_render_graph_edges(
                Core2d,
                (
                    Node2d::Tonemapping,
                    PxRender,
                    Node2d::EndMainPassPostProcessing,
                ),
            )
            .init_resource::<PxUniformBuffer>()
            .add_systems(Render, prepare_uniform.in_set(RenderSet::Prepare));
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp).init_resource::<PxPipeline>();
    }
}

/// Size of the image which `seldom_pixel` draws to
#[derive(Clone, Copy, Debug)]
pub enum ScreenSize {
    /// The screen will have the given dimensions, which is scaled up to fit the window, preserving
    /// the given dimensions' aspect ratio
    Fixed(UVec2),
    /// The screen will match the aspect ratio of the window, with an area of at least as many
    /// pixels as given
    MinPixels(u32),
}

impl From<UVec2> for ScreenSize {
    fn from(value: UVec2) -> Self {
        Self::Fixed(value)
    }
}

impl ScreenSize {
    fn compute(self, window_size: Vec2) -> UVec2 {
        use ScreenSize::*;

        match self {
            Fixed(size) => size,
            MinPixels(pixels) => {
                let pixels = pixels as f32;
                let width = (window_size.x * pixels / window_size.y).sqrt();
                let height = pixels / width;

                UVec2::new(width as u32, height as u32)
            }
        }
    }
}

/// Metadata for the image that `seldom_pixel` draws to
#[derive(ExtractResource, Resource, Clone, Debug)]
pub struct Screen {
    pub(crate) size: ScreenSize,
    pub(crate) computed_size: UVec2,
    window_aspect_ratio: f32,
    pub(crate) palette: [Vec3; 256],
}

impl Screen {
    /// Computed size of the screen
    pub fn size(&self) -> UVec2 {
        self.computed_size
    }
}

pub(crate) fn screen_scale(screen_size: UVec2, window_size: Vec2) -> Vec2 {
    let aspect = screen_size.y as f32 / screen_size.x as f32;

    Vec2::from(match window_size.y > aspect * window_size.x {
        true => (window_size.x, window_size.x * aspect),
        false => (window_size.y / aspect, window_size.y),
    })
}

fn insert_screen(size: ScreenSize) -> impl Fn(Query<&Window, With<PrimaryWindow>>, Commands) {
    move |windows, mut commands| {
        let window = windows.single();

        commands.insert_resource(Screen {
            size,
            computed_size: size.compute(Vec2::new(window.width(), window.height())),
            window_aspect_ratio: window.width() / window.height(),
            palette: [Vec3::ZERO; 256],
        });
    }
}

fn init_screen(mut initialized: Local<bool>, palette: PaletteParam, mut screen: ResMut<Screen>) {
    if *initialized {
        return;
    }

    let Some(palette) = palette.get() else {
        return;
    };

    let mut screen_palette = [Vec3::ZERO; 256];

    for (i, [r, g, b]) in palette.colors.iter().enumerate() {
        screen_palette[i] = Color::srgb_u8(*r, *g, *b).to_linear().to_vec3();
    }

    screen.palette = screen_palette;

    *initialized = false;
}

fn resize_screen(mut window_resized: EventReader<WindowResized>, mut screen: ResMut<Screen>) {
    if let Some(window_resized) = window_resized.read().last() {
        screen.computed_size = screen
            .size
            .compute(Vec2::new(window_resized.width, window_resized.height));
        screen.window_aspect_ratio = window_resized.width / window_resized.height;
    }
}

#[derive(ShaderType)]
struct PxUniform {
    palette: [Vec3; 256],
    fit_factor: Vec2,
}

#[derive(Resource, Deref, DerefMut, Default)]
struct PxUniformBuffer(DynamicUniformBuffer<PxUniform>);

fn prepare_uniform(
    mut buffer: ResMut<PxUniformBuffer>,
    screen: Res<Screen>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    let Some(mut writer) = buffer.get_writer(1, &device, &queue) else {
        return;
    };

    let aspect_ratio_ratio =
        screen.computed_size.x as f32 / screen.computed_size.y as f32 / screen.window_aspect_ratio;
    writer.write(&PxUniform {
        palette: screen.palette,
        fit_factor: if aspect_ratio_ratio > 1. {
            Vec2::new(1., 1. / aspect_ratio_ratio)
        } else {
            Vec2::new(aspect_ratio_ratio, 1.)
        },
    });
}

#[derive(Resource)]
struct PxPipeline {
    layout: BindGroupLayout,
    id: CachedRenderPipelineId,
}

impl FromWorld for PxPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "px_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Uint),
                    uniform_buffer::<PxUniform>(false).visibility(ShaderStages::VERTEX_FRAGMENT),
                ),
            ),
        );

        Self {
            id: world.resource_mut::<PipelineCache>().queue_render_pipeline(
                RenderPipelineDescriptor {
                    label: Some("px_pipeline".into()),
                    layout: vec![layout.clone()],
                    vertex: VertexState {
                        shader: SCREEN_SHADER_HANDLE,
                        shader_defs: Vec::new(),
                        entry_point: "vertex".into(),
                        buffers: Vec::new(),
                    },
                    fragment: Some(FragmentState {
                        shader: SCREEN_SHADER_HANDLE,
                        shader_defs: Vec::new(),
                        entry_point: "fragment".into(),
                        targets: vec![Some(ColorTargetState {
                            format: TextureFormat::bevy_default(),
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive: default(),
                    depth_stencil: None,
                    multisample: default(),
                    push_constant_ranges: Vec::new(),
                },
            ),
            layout,
        }
    }
}

#[derive(RenderLabel, Hash, Eq, PartialEq, Clone, Debug)]
struct PxRender;

struct PxRenderNode<L: PxLayer> {
    maps: QueryState<MapComponents<L>>,
    tiles: QueryState<TileComponents>,
    frames: QueryState<FrameComponents<L>>,
    sprites: QueryState<SpriteComponents<L>>,
    texts: QueryState<TextComponents<L>>,
    #[cfg(feature = "line")]
    lines: QueryState<LineComponents<L>>,
    filters: QueryState<FilterComponents<L>, Without<PxCanvas>>,
}

impl<L: PxLayer> FromWorld for PxRenderNode<L> {
    fn from_world(world: &mut World) -> Self {
        Self {
            maps: world.query(),
            tiles: world.query(),
            frames: world.query(),
            sprites: world.query(),
            texts: world.query(),
            #[cfg(feature = "line")]
            lines: world.query(),
            filters: world.query_filtered(),
        }
    }
}

impl<L: PxLayer> ViewNode for PxRenderNode<L> {
    type ViewQuery = &'static ViewTarget;

    fn update(&mut self, world: &mut World) {
        self.maps.update_archetypes(world);
        self.tiles.update_archetypes(world);
        self.frames.update_archetypes(world);
        self.sprites.update_archetypes(world);
        self.texts.update_archetypes(world);
        #[cfg(feature = "line")]
        self.lines.update_archetypes(world);
        self.filters.update_archetypes(world);
    }

    fn run<'w>(
        &self,
        _: &mut RenderGraphContext,
        render_context: &mut RenderContext<'w>,
        target: &ViewTarget,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let &camera = world.resource::<PxCamera>();
        let &LastUpdate(last_update) = world.resource::<LastUpdate>();
        let screen = world.resource::<Screen>();

        let mut image = Image::new_fill(
            Extent3d {
                width: screen.computed_size.x,
                height: screen.computed_size.y,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0],
            TextureFormat::R8Uint,
            default(),
        );

        #[cfg(feature = "line")]
        let mut layer_contents = BTreeMap::<
            _,
            (
                Vec<_>,
                Vec<_>,
                Vec<_>,
                Vec<_>,
                Vec<_>,
                Vec<_>,
                Vec<_>,
                Vec<_>,
            ),
        >::default();
        #[cfg(not(feature = "line"))]
        let mut layer_contents =
            BTreeMap::<_, (Vec<_>, Vec<_>, Vec<_>, Vec<_>, (), Vec<_>, (), Vec<_>)>::default();

        for (map, tileset, position, layer, canvas, animation, filter) in
            self.maps.iter_manual(world)
        {
            if let Some((maps, _, _, _, _, _, _, _)) = layer_contents.get_mut(layer) {
                maps.push((map, tileset, position, canvas, animation, filter));
            } else {
                layer_contents.insert(
                    layer.clone(),
                    (
                        vec![(map, tileset, position, canvas, animation, filter)],
                        default(),
                        default(),
                        default(),
                        default(),
                        default(),
                        default(),
                        default(),
                    ),
                );
            }
        }

        for (sprite, position, anchor, layer, canvas, animation, filter) in
            self.sprites.iter_manual(world)
        {
            if let Some((_, sprites, _, _, _, _, _, _)) = layer_contents.get_mut(layer) {
                sprites.push((sprite, position, anchor, canvas, animation, filter));
            } else {
                layer_contents.insert(
                    layer.clone(),
                    (
                        default(),
                        vec![(sprite, position, anchor, canvas, animation, filter)],
                        default(),
                        default(),
                        default(),
                        default(),
                        default(),
                        default(),
                    ),
                );
            }
        }

        for (frame, position, offset, size, anchor, layer, canvas, filter) in
            self.frames.iter_manual(world)
        {
            if let Some((_, _, frames, _, _, _, _, _)) = layer_contents.get_mut(layer) {
                frames.push((frame, position, offset, size, anchor, canvas, filter));
            } else {
                layer_contents.insert(
                    layer.clone(),
                    (
                        default(),
                        default(),
                        vec![(frame, position, offset, size, anchor, canvas, filter)],
                        default(),
                        default(),
                        default(),
                        default(),
                        default(),
                    ),
                );
            }
        }

        for (text, typeface, rect, alignment, layer, canvas, animation, filter) in
            self.texts.iter_manual(world)
        {
            if let Some((_, _, _, texts, _, _, _, _)) = layer_contents.get_mut(layer) {
                texts.push((text, typeface, rect, alignment, canvas, animation, filter));
            } else {
                layer_contents.insert(
                    layer.clone(),
                    (
                        default(),
                        default(),
                        default(),
                        vec![(text, typeface, rect, alignment, canvas, animation, filter)],
                        default(),
                        default(),
                        default(),
                        default(),
                    ),
                );
            }
        }

        #[cfg(feature = "line")]
        for (line, filter, layers, canvas, animation) in self.lines.iter_manual(world) {
            for (layer, clip) in match layers {
                PxFilterLayers::Single { layer, clip } => vec![(layer.clone(), *clip)],
                PxFilterLayers::Many(layers) => {
                    layers.iter().map(|layer| (layer.clone(), true)).collect()
                }
                PxFilterLayers::Select(select_fn) => layer_contents
                    .keys()
                    .filter(|layer| select_fn(layer))
                    .map(|layer| (layer.clone(), true))
                    .collect(),
            }
            .into_iter()
            {
                if let Some((_, _, _, clip_lines, _, over_lines, _)) =
                    layer_contents.get_mut(&layer)
                {
                    if clip { clip_lines } else { over_lines }
                        .push((line, filter, canvas, animation));
                } else {
                    let lines = vec![(line, filter, canvas, animation)];

                    layer_contents.insert(
                        layer,
                        if clip {
                            (
                                default(),
                                default(),
                                default(),
                                default(),
                                lines,
                                default(),
                                default(),
                                default(),
                            )
                        } else {
                            (
                                default(),
                                default(),
                                default(),
                                default(),
                                default(),
                                default(),
                                lines,
                                default(),
                            )
                        },
                    );
                }
            }
        }

        let tilesets = world.resource::<RenderAssets<PxTileset>>();
        let frame_assets = world.resource::<RenderAssets<PxFrame>>();
        let sprite_assets = world.resource::<RenderAssets<PxSprite>>();
        let typefaces = world.resource::<RenderAssets<PxTypeface>>();
        let filters = world.resource::<RenderAssets<PxFilter>>();

        for (filter, layers, animation) in self.filters.iter_manual(world) {
            for (layer, clip) in match layers {
                PxFilterLayers::Single { layer, clip } => vec![(layer.clone(), *clip)],
                PxFilterLayers::Many(layers) => {
                    layers.iter().map(|layer| (layer.clone(), true)).collect()
                }
                PxFilterLayers::Select(select_fn) => layer_contents
                    .keys()
                    .filter(|layer| select_fn(layer))
                    .map(|layer| (layer.clone(), true))
                    .collect(),
            }
            .into_iter()
            {
                if let Some((_, _, _, _, _, clip_filters, _, over_filters)) =
                    layer_contents.get_mut(&layer)
                {
                    if clip { clip_filters } else { over_filters }.push((filter, animation));
                } else {
                    let filters = vec![(filter, animation)];

                    layer_contents.insert(
                        layer,
                        if clip {
                            (
                                default(),
                                default(),
                                default(),
                                default(),
                                default(),
                                filters,
                                default(),
                                default(),
                            )
                        } else {
                            (
                                default(),
                                default(),
                                default(),
                                default(),
                                default(),
                                default(),
                                default(),
                                filters,
                            )
                        },
                    );
                }
            }
        }

        let mut layer_image = PxImage::<Option<u8>>::empty_from_image(&image);
        let mut image_slice = PxImageSliceMut::from_image_mut(&mut image);

        #[allow(unused_variables)]
        for (
            _,
            (maps, sprites, frames, texts, clip_lines, clip_filters, over_lines, over_filters),
        ) in layer_contents.into_iter()
        {
            layer_image.clear();

            for (map, tileset, position, canvas, animation, map_filter) in maps {
                let Some(tileset) = tilesets.get(tileset) else {
                    continue;
                };

                let map_filter = map_filter.and_then(|map_filter| filters.get(map_filter));
                let size = map.size();

                for x in 0..size.x {
                    for y in 0..size.y {
                        let pos = UVec2::new(x, y);
                        let Some(tile) = map.get(pos) else {
                            continue;
                        };

                        let Ok((&PxTile { texture }, tile_filter)) =
                            self.tiles.get_manual(world, tile)
                        else {
                            continue;
                        };

                        let Some(tile) = tileset.tileset.get(texture as usize) else {
                            error!("tile texture index out of bounds: the len is {}, but the index is {texture}", tileset.tileset.len());
                            continue;
                        };

                        draw_spatial(
                            tile,
                            (),
                            &mut layer_image,
                            (**position + pos.as_ivec2() * tileset.tile_size().as_ivec2()).into(),
                            PxAnchor::BottomLeft,
                            *canvas,
                            copy_animation_params(animation, last_update),
                            [
                                tile_filter.and_then(|tile_filter| filters.get(tile_filter)),
                                map_filter,
                            ]
                            .into_iter()
                            .flatten(),
                            camera,
                        );
                    }
                }
            }

            for (frame, position, offset, size, anchor, canvas, filter) in frames {
                let Some(frame) = frame_assets.get(frame) else {
                    continue;
                };

                draw_frame(
                    frame,
                    (),
                    &mut layer_image,
                    *position,
                    *offset,
                    *size,
                    *anchor,
                    *canvas,
                    filter.and_then(|filter| filters.get(filter)),
                    camera,
                );
            }

            for (sprite, position, anchor, canvas, animation, filter) in sprites {
                let Some(sprite) = sprite_assets.get(sprite) else {
                    continue;
                };

                draw_spatial(
                    sprite,
                    (),
                    &mut layer_image,
                    *position,
                    *anchor,
                    *canvas,
                    copy_animation_params(animation, last_update),
                    filter.and_then(|filter| filters.get(filter)),
                    camera,
                );
            }

            for (text, typeface, rect, alignment, canvas, animation, filter) in texts {
                let Some(typeface) = typefaces.get(typeface) else {
                    continue;
                };

                let rect = match canvas {
                    PxCanvas::World => rect.sub_ivec2(*camera),
                    PxCanvas::Camera => **rect,
                };
                let rect_size = rect.size().as_uvec2();
                let line_count = (rect_size.y + 1) / (typeface.height + 1);

                let mut lines = Vec::default();
                let mut line = Vec::default();
                let mut line_width = 0;
                let mut word = Vec::default();
                let mut word_width = 0;
                let mut separator = Vec::default();
                let mut separator_width = 0;
                for character in text.chars() {
                    let (character_width, is_separator) = typeface
                        .characters
                        .get(&character)
                        .map(|character| (character.data.width() as u32, false))
                        .unwrap_or_else(|| {
                            (
                                typeface
                                    .separators
                                    .get(&character)
                                    .map(|separator| separator.width)
                                    .unwrap_or_else(|| {
                                        error!(
                                            "received character '{character}' that isn't in typeface"
                                        );
                                        0
                                    }),
                                true,
                            )
                        });

                    if if is_separator {
                        if line_width + separator_width + word_width - 1 > rect_size.x {
                            lines.push((line_width, line));
                            line_width = word_width - 1;
                            line = word;
                            word_width = 0;
                            word = default();
                            separator_width = character_width;
                            separator = vec![character];
                            true
                        } else if word.is_empty() {
                            separator_width += character_width;
                            separator.push(character);
                            false
                        } else {
                            line_width += separator_width + word_width - 1;
                            line.append(&mut separator);
                            line.append(&mut word);
                            word_width = 0;
                            separator_width = character_width;
                            separator = vec![character];
                            false
                        }
                    } else if word_width + character_width > rect_size.x {
                        if !line.is_empty() {
                            lines.push((line_width, line));
                            line_width = 0;
                            line = default();
                        }

                        if word_width > 0 {
                            lines.push((word_width - 1, word));
                        }
                        word_width = character_width + 1;
                        word = vec![character];
                        separator_width = 0;
                        separator = default();
                        true
                    } else {
                        word_width += character_width + 1;
                        word.push(character);
                        false
                    } && lines.len() as u32 > line_count
                    {
                        line_width = 0;
                        line.clear();
                        word_width = 0;
                        word.clear();
                        separator_width = 0;
                        separator.clear();
                        break;
                    }
                }

                if line_width + separator_width + word_width + 1 > rect_size.x {
                    lines.push((line_width, line));
                    if word_width > 0 {
                        lines.push((word_width - 1, word));
                    }
                } else if !word.is_empty() {
                    line_width += separator_width + word_width - 1;
                    line.append(&mut separator);
                    line.append(&mut word);
                    lines.push((line_width, line));
                }

                if lines.len() as u32 > line_count {
                    for _ in 0..lines.len() as u32 - line_count {
                        lines.pop();
                    }
                }

                let mut text_image = PxImage::empty(rect_size);
                let lines_height =
                    (lines.len() as u32 * typeface.height + lines.len() as u32).max(1) - 1;
                let mut line_y = alignment.y_pos(rect_size.y - lines_height)
                    + lines.len() as u32 * (typeface.height + 1);

                for (line_width, line) in lines {
                    line_y -= typeface.height + 1;
                    let mut character_x = alignment.x_pos(rect_size.x - line_width);
                    let mut was_character = false;

                    for character in line {
                        character_x += if let Some(character) = typeface.characters.get(&character)
                        {
                            was_character = true;

                            draw_spatial(
                                character,
                                (),
                                &mut text_image,
                                IVec2::new(character_x as i32, line_y as i32).into(),
                                PxAnchor::BottomLeft,
                                PxCanvas::Camera,
                                copy_animation_params(animation, last_update),
                                filter.and_then(|filter| filters.get(filter)),
                                camera,
                            );

                            character.data.width() as u32 + 1
                        } else {
                            if was_character {
                                character_x -= 1;
                            }
                            was_character = false;

                            typeface.separators.get(&character).unwrap().width
                        };
                    }
                }

                if let Some(filter) = filter {
                    if let Some(PxFilter(filter)) = filters.get(filter) {
                        text_image.slice_all_mut().for_each_mut(|_, _, pixel| {
                            if let Some(pixel) = pixel {
                                *pixel = filter.pixel(IVec2::new(*pixel as i32, 0));
                            }
                        });
                    }
                }

                layer_image.slice_mut(rect).draw(&text_image);
            }

            // This is where I draw the line! /j
            #[cfg(feature = "line")]
            for (line, filter, canvas, animation) in clip_lines {
                if let Some(filter) = filters.get(filter) {
                    draw_line(
                        line,
                        filter,
                        &mut layer_image.slice_all_mut(),
                        *canvas,
                        copy_animation_params(animation, last_update),
                        camera,
                    );
                }
            }

            for (filter, animation) in clip_filters {
                if let Some(filter) = filters.get(filter) {
                    draw_filter(
                        filter,
                        copy_animation_params(animation, last_update),
                        &mut layer_image.slice_all_mut(),
                    );
                }
            }

            image_slice.draw(&layer_image);

            #[cfg(feature = "line")]
            for (line, filter, canvas, animation) in over_lines {
                if let Some(filter) = filters.get(filter) {
                    draw_line(
                        line,
                        filter,
                        &mut image_slice,
                        *canvas,
                        copy_animation_params(animation, last_update),
                        camera,
                    );
                }
            }

            for (filter, animation) in over_filters {
                if let Some(filter) = filters.get(filter) {
                    draw_filter(
                        filter,
                        copy_animation_params(animation, last_update),
                        &mut image_slice,
                    );
                }
            }
        }

        let cursor = world.resource::<CursorState>();

        if let PxCursor::Filter {
            idle,
            left_click,
            right_click,
        } = world.resource()
        {
            if let Some(cursor_pos) = **world.resource::<PxCursorPosition>() {
                if let Some(PxFilter(filter)) = filters.get(match cursor {
                    CursorState::Idle => idle,
                    CursorState::Left => left_click,
                    CursorState::Right => right_click,
                }) {
                    let mut image = PxImageSliceMut::from_image_mut(&mut image);

                    if let Some(pixel) = image.get_pixel_mut(IVec2::new(
                        cursor_pos.x as i32,
                        image.height() as i32 - 1 - cursor_pos.y as i32,
                    )) {
                        *pixel = filter
                            .get_pixel(IVec2::new(*pixel as i32, 0))
                            .expect("filter is incorrect size");
                    }
                }
            }
        }

        let Some(uniform_binding) = world.resource::<PxUniformBuffer>().binding() else {
            return Ok(());
        };

        let texture = render_context
            .render_device()
            .create_texture(&image.texture_descriptor);

        world.resource::<RenderQueue>().write_texture(
            texture.as_image_copy(),
            &image.data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(
                    image.width() * image.texture_descriptor.format.pixel_size() as u32,
                ),
                rows_per_image: None,
            },
            image.texture_descriptor.size,
        );

        let texture_view = texture.create_view(&TextureViewDescriptor {
            label: Some("px_texture_view"),
            format: Some(image.texture_descriptor.format),
            dimension: Some(TextureViewDimension::D2),
            ..default()
        });

        let px_pipeline = world.resource::<PxPipeline>();
        let Some(pipeline) = world
            .resource::<PipelineCache>()
            .get_render_pipeline(px_pipeline.id)
        else {
            return Ok(());
        };

        let post_process = target.post_process_write();

        let bind_group = render_context.render_device().create_bind_group(
            "px_bind_group",
            &px_pipeline.layout,
            &BindGroupEntries::sequential((&texture_view, uniform_binding.clone())),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("px_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: post_process.destination,
                resolve_target: None,
                ops: default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.draw(0..6, 0..1);

        Ok(())
    }
}

fn update_screen_palette(
    mut waiting_for_load: Local<bool>,
    palette_handle: Res<PaletteHandle>,
    mut screen: ResMut<Screen>,
    palette: PaletteParam,
) {
    if !palette_handle.is_changed() && !*waiting_for_load {
        return;
    }

    let Some(palette) = palette.get() else {
        *waiting_for_load = true;
        return;
    };

    let mut screen_palette = [Vec3::ZERO; 256];

    for (i, [r, g, b]) in palette.colors.iter().enumerate() {
        screen_palette[i] = Color::srgb_u8(*r, *g, *b).to_linear().to_vec3();
    }

    screen.palette = screen_palette;

    *waiting_for_load = false;
}

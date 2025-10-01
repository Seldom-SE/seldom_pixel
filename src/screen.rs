//! Screen and rendering

// TODO Split out a module

use std::{collections::BTreeMap, iter::empty, marker::PhantomData};

use bevy_asset::uuid_handle;
use bevy_core_pipeline::core_2d::graph::{Core2d, Node2d};
use bevy_derive::{Deref, DerefMut};
use bevy_image::TextureFormatPixelInfo;
use bevy_math::{ivec2, uvec2};
use bevy_render::{
    Render, RenderApp, RenderSystems,
    extract_resource::{ExtractResource, ExtractResourcePlugin},
    render_asset::RenderAssets,
    render_graph::{
        NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
    },
    render_resource::{
        BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
        ColorTargetState, ColorWrites, DynamicUniformBuffer, Extent3d, FragmentState,
        PipelineCache, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
        ShaderStages, ShaderType, TexelCopyBufferLayout, TextureDimension, TextureFormat,
        TextureSampleType, TextureViewDescriptor, TextureViewDimension, VertexState,
        binding_types::{texture_2d, uniform_buffer},
    },
    renderer::{RenderContext, RenderDevice, RenderQueue},
    view::ViewTarget,
};
use bevy_window::{PrimaryWindow, WindowResized};

#[cfg(feature = "line")]
use crate::line::{LineComponents, draw_line};
use crate::{
    animation::draw_spatial,
    cursor::{CursorState, PxCursorPosition},
    filter::{FilterComponents, draw_filter},
    image::{PxImage, PxImageSliceMut},
    map::{MapComponents, PxTile, TileComponents},
    palette::{Palette, PaletteHandle},
    position::PxLayer,
    prelude::*,
    rect::RectComponents,
    sprite::SpriteComponents,
    text::TextComponents,
};

const SCREEN_SHADER_HANDLE: Handle<Shader> = uuid_handle!("48CE4F2C-8B78-5954-08A8-461F62E10E84");

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
        // R-A workaround
        Assets::insert(
            &mut app
                .add_plugins(ExtractResourcePlugin::<Screen>::default())
                .add_systems(Startup, insert_screen(self.size))
                .add_systems(Update, init_screen)
                .add_systems(PostUpdate, (resize_screen, update_screen_palette))
                .world_mut()
                .resource_mut::<Assets<Shader>>(),
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
            .add_systems(Render, prepare_uniform.in_set(RenderSystems::Prepare));
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
    // pub(crate) palette_tree: ImmutableKdTree<f32, 3>,
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

fn insert_screen(
    size: ScreenSize,
) -> impl Fn(Query<&Window, With<PrimaryWindow>>, Commands) -> Result<()> {
    move |windows, mut commands| {
        let window = windows.single()?;

        commands.insert_resource(Screen {
            size,
            computed_size: size.compute(Vec2::new(window.width(), window.height())),
            window_aspect_ratio: window.width() / window.height(),
            palette: [Vec3::ZERO; 256],
            // palette_tree: ImmutableKdTree::from(&[][..]),
        });

        OK
    }
}

fn init_screen(
    mut initialized: Local<bool>,
    palette: Res<PaletteHandle>,
    palettes: Res<Assets<Palette>>,
    mut screen: ResMut<Screen>,
) {
    if *initialized {
        return;
    }

    let Some(palette) = palettes.get(&**palette) else {
        return;
    };

    let mut screen_palette = [Vec3::ZERO; 256];

    for (i, [r, g, b]) in palette.colors.iter().enumerate() {
        screen_palette[i] = Color::srgb_u8(*r, *g, *b).to_linear().to_vec3();
    }

    screen.palette = screen_palette;

    *initialized = false;
}

fn resize_screen(mut window_resized: MessageReader<WindowResized>, mut screen: ResMut<Screen>) {
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
                        entry_point: Some("vertex".into()),
                        buffers: Vec::new(),
                    },
                    fragment: Some(FragmentState {
                        shader: SCREEN_SHADER_HANDLE,
                        shader_defs: Vec::new(),
                        entry_point: Some("fragment".into()),
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
                    zero_initialize_workgroup_memory: true,
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
    // image_to_sprites: QueryState<ImageToSpriteComponents<L>>,
    sprites: QueryState<SpriteComponents<L>>,
    texts: QueryState<TextComponents<L>>,
    rects: QueryState<RectComponents<L>>,
    #[cfg(feature = "line")]
    lines: QueryState<LineComponents<L>>,
    filters: QueryState<FilterComponents<L>, Without<PxCanvas>>,
}

impl<L: PxLayer> FromWorld for PxRenderNode<L> {
    fn from_world(world: &mut World) -> Self {
        Self {
            maps: world.query(),
            tiles: world.query(),
            // image_to_sprites: world.query(),
            sprites: world.query(),
            texts: world.query(),
            rects: world.query(),
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
        // self.image_to_sprites.update_archetypes(world);
        self.sprites.update_archetypes(world);
        self.texts.update_archetypes(world);
        self.rects.update_archetypes(world);
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
                Vec<_>,
            ),
        >::default();
        #[cfg(not(feature = "line"))]
        let mut layer_contents = BTreeMap::<
            _,
            (
                Vec<_>,
                Vec<_>,
                Vec<_>,
                Vec<_>,
                (),
                Vec<_>,
                Vec<_>,
                (),
                Vec<_>,
            ),
        >::default();

        for (map, &pos, layer, &canvas, animation, filter) in self.maps.iter_manual(world) {
            let map = (map, pos, canvas, animation, filter);

            if let Some((maps, _, _, _, _, _, _, _, _)) = layer_contents.get_mut(layer) {
                maps.push(map);
            } else {
                BTreeMap::insert(
                    &mut layer_contents,
                    layer.clone(),
                    (
                        vec![map],
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
                        default(),
                        Vec::new(),
                        Vec::new(),
                        default(),
                        Vec::new(),
                    ),
                );
            }
        }

        // for (image, position, anchor, layer, canvas, filter) in
        //     self.image_to_sprites.iter_manual(world)
        // {
        //     if let Some((_, image_to_sprites, _, _, _, _, _, _)) = layer_contents.get_mut(layer) {
        //         image_to_sprites.push((image, position, anchor, canvas, filter));
        //     } else {
        //         layer_contents.insert(
        //             layer.clone(),
        //             (
        //                 default(),
        //                 vec![(image, position, anchor, canvas, filter)],
        //                 default(),
        //                 default(),
        //                 default(),
        //                 default(),
        //                 default(),
        //                 default(),
        //             ),
        //         );
        //     }
        // }

        for (sprite, &position, &anchor, layer, &canvas, animation, filter) in
            self.sprites.iter_manual(world)
        {
            let sprite = (sprite, position, anchor, canvas, animation, filter);

            if let Some((_, sprites, _, _, _, _, _, _, _)) = layer_contents.get_mut(layer) {
                sprites.push(sprite);
            } else {
                BTreeMap::insert(
                    &mut layer_contents,
                    layer.clone(),
                    (
                        Vec::new(),
                        vec![sprite],
                        Vec::new(),
                        Vec::new(),
                        default(),
                        Vec::new(),
                        Vec::new(),
                        default(),
                        Vec::new(),
                    ),
                );
            }
        }

        for (text, &pos, &alignment, layer, &canvas, animation, filter) in
            self.texts.iter_manual(world)
        {
            let text = (text, pos, alignment, canvas, animation, filter);

            if let Some((_, _, texts, _, _, _, _, _, _)) = layer_contents.get_mut(layer) {
                texts.push(text);
            } else {
                BTreeMap::insert(
                    &mut layer_contents,
                    layer.clone(),
                    (
                        Vec::new(),
                        Vec::new(),
                        vec![text],
                        Vec::new(),
                        default(),
                        Vec::new(),
                        Vec::new(),
                        default(),
                        Vec::new(),
                    ),
                );
            }
        }

        for (&rect, filter, layers, &pos, &anchor, &canvas, animation, invert) in
            self.rects.iter_manual(world)
        {
            for (layer, clip) in match layers {
                PxFilterLayers::Single { layer, clip } => vec![(layer.clone(), *clip)],
                // TODO Need to do this after all layers have been extracted
                PxFilterLayers::Range(range) => layer_contents
                    .keys()
                    .filter(|layer| range.contains(layer))
                    .map(|layer| (layer.clone(), true))
                    .collect(),
                PxFilterLayers::Many(layers) => {
                    layers.iter().map(|layer| (layer.clone(), true)).collect()
                }
            }
            .into_iter()
            {
                let rect = (rect, filter, pos, anchor, canvas, animation, invert);

                if let Some((_, _, _, clip_rects, _, _, over_rects, _, _)) =
                    layer_contents.get_mut(&layer)
                {
                    if clip { clip_rects } else { over_rects }.push(rect);
                } else {
                    let rects = vec![rect];

                    BTreeMap::insert(
                        &mut layer_contents,
                        layer,
                        if clip {
                            (
                                Vec::new(),
                                Vec::new(),
                                Vec::new(),
                                rects,
                                default(),
                                Vec::new(),
                                Vec::new(),
                                default(),
                                Vec::new(),
                            )
                        } else {
                            (
                                Vec::new(),
                                Vec::new(),
                                Vec::new(),
                                Vec::new(),
                                default(),
                                Vec::new(),
                                rects,
                                default(),
                                Vec::new(),
                            )
                        },
                    );
                }
            }
        }

        #[cfg(feature = "line")]
        for (line, filter, layers, &canvas, animation, invert) in self.lines.iter_manual(world) {
            let line = (line, filter, canvas, animation, invert);

            for (layer, clip) in match layers {
                PxFilterLayers::Single { layer, clip } => vec![(layer.clone(), *clip)],
                PxFilterLayers::Range(range) => layer_contents
                    .keys()
                    .filter(|layer| range.contains(layer))
                    .map(|layer| (layer.clone(), true))
                    .collect(),
                PxFilterLayers::Many(layers) => {
                    layers.iter().map(|layer| (layer.clone(), true)).collect()
                }
            }
            .into_iter()
            {
                if let Some((_, _, _, _, clip_lines, _, _, over_lines, _)) =
                    layer_contents.get_mut(&layer)
                {
                    if clip { clip_lines } else { over_lines }.push(line);
                } else {
                    let lines = vec![line];

                    BTreeMap::insert(
                        &mut layer_contents,
                        layer,
                        if clip {
                            (
                                Vec::new(),
                                Vec::new(),
                                Vec::new(),
                                Vec::new(),
                                lines,
                                Vec::new(),
                                Vec::new(),
                                Vec::new(),
                                Vec::new(),
                            )
                        } else {
                            (
                                Vec::new(),
                                Vec::new(),
                                Vec::new(),
                                Vec::new(),
                                Vec::new(),
                                Vec::new(),
                                Vec::new(),
                                lines,
                                Vec::new(),
                            )
                        },
                    );
                }
            }
        }

        for (filter, layers, animation) in self.filters.iter_manual(world) {
            let filter = (filter, animation);

            for (layer, clip) in match layers {
                PxFilterLayers::Single { layer, clip } => vec![(layer.clone(), *clip)],
                PxFilterLayers::Range(range) => layer_contents
                    .keys()
                    .filter(|layer| range.contains(layer))
                    .map(|layer| (layer.clone(), true))
                    .collect(),
                PxFilterLayers::Many(layers) => {
                    layers.iter().map(|layer| (layer.clone(), true)).collect()
                }
            }
            .into_iter()
            {
                if let Some((_, _, _, _, _, clip_filters, _, _, over_filters)) =
                    layer_contents.get_mut(&layer)
                {
                    if clip { clip_filters } else { over_filters }.push(filter);
                } else {
                    let filters = vec![filter];

                    BTreeMap::insert(
                        &mut layer_contents,
                        layer,
                        if clip {
                            (
                                Vec::new(),
                                Vec::new(),
                                Vec::new(),
                                Vec::new(),
                                default(),
                                filters,
                                Vec::new(),
                                default(),
                                Vec::new(),
                            )
                        } else {
                            (
                                Vec::new(),
                                Vec::new(),
                                Vec::new(),
                                Vec::new(),
                                default(),
                                Vec::new(),
                                Vec::new(),
                                default(),
                                filters,
                            )
                        },
                    );
                }
            }
        }

        let tilesets = world.resource::<RenderAssets<PxTileset>>();
        // let images = world.resource::<RenderAssets<GpuImage>>();
        let sprite_assets = world.resource::<RenderAssets<PxSpriteAsset>>();
        let typefaces = world.resource::<RenderAssets<PxTypeface>>();
        let filters = world.resource::<RenderAssets<PxFilterAsset>>();

        let mut layer_image = PxImage::empty_from_image(&image);
        let mut image_slice = PxImageSliceMut::from_image_mut(&mut image).unwrap();

        #[allow(unused_variables)]
        for (
            _,
            (
                maps,
                // image_to_sprites,
                sprites,
                texts,
                clip_rects,
                clip_lines,
                clip_filters,
                over_rects,
                over_lines,
                over_filters,
            ),
        ) in layer_contents.into_iter()
        {
            layer_image.clear();
            let mut layer_slice = layer_image.slice_all_mut();

            for (map, position, canvas, frame, map_filter) in maps {
                let Some(tileset) = tilesets.get(&map.tileset) else {
                    continue;
                };

                let map_filter = map_filter.and_then(|map_filter| filters.get(&**map_filter));
                let size = map.tiles.size();

                for x in 0..size.x {
                    for y in 0..size.y {
                        let pos = UVec2::new(x, y);

                        let Some(tile) = map.tiles.get(pos) else {
                            continue;
                        };

                        let Ok((&PxTile { texture }, tile_filter)) =
                            self.tiles.get_manual(world, tile)
                        else {
                            continue;
                        };

                        let Some(tile) = tileset.tileset.get(texture as usize) else {
                            error!(
                                "tile texture index out of bounds: the len is {}, but the index is {texture}",
                                tileset.tileset.len()
                            );
                            continue;
                        };

                        draw_spatial(
                            tile,
                            (),
                            &mut layer_slice,
                            (*position + pos.as_ivec2() * tileset.tile_size().as_ivec2()).into(),
                            PxAnchor::BottomLeft,
                            canvas,
                            frame.copied(),
                            [
                                tile_filter.and_then(|tile_filter| filters.get(&**tile_filter)),
                                map_filter,
                            ]
                            .into_iter()
                            .flatten(),
                            camera,
                        );
                    }
                }
            }

            // I was trying to make `ImageToSprite` work without 1-frame lag, but this
            // fundamentally needs GPU readback or something bc you can't just get image data
            // from a `GpuImage`. I think those represent images that're actually on the GPU. So
            // here's where I left off with that. I don't need `ImageToSprite` at the moment, so
            // this will be left incomplete until I need it, if I ever do.

            // // TODO Use more helpers
            // // TODO Feature gate
            // // TODO Immediate function version
            // for (image, position, anchor, canvas, filter) in image_to_sprites {
            //     // let palette = screen.palette
            //     //     .colors
            //     //     .iter()
            //     //     .map(|&color| Oklaba::from(Srgba::from_u8_array_no_alpha(color)).to_vec3())
            //     //     .collect::<Vec<Vec3>>();

            //     let palette_tree = ImmutableKdTree::from(
            //         &screen
            //             .palette
            //             .iter()
            //             .map(|&color| color.into())
            //             .collect::<Vec<[f32; 3]>>()[..],
            //     );

            //     let dither = &image.dither;
            //     let Some(image) = images.get(&image.image) else {
            //         continue;
            //     };

            //     // TODO https://github.com/bevyengine/bevy/blob/v0.14.1/examples/app/headless_renderer.rs
            //     let size = image.size;
            //     let data = PxImage::empty(size);

            //     let mut sprite = PxSprite {
            //         frame_size: data.area(),
            //         data,
            //     };

            //     let mut pixels = image
            //         .data
            //         .chunks_exact(4)
            //         .zip(sprite.data.iter_mut())
            //         .enumerate()
            //         .collect::<Vec<_>>();

            //     pixels.par_chunk_map_mut(ComputeTaskPool::get(), 20, |_, pixels| {
            //         use DitherAlgorithm::*;
            //         use ThresholdMap::*;

            //         match *dither {
            //             None => dither_slice::<ClosestAlg, 1>(
            //                 pixels,
            //                 0.,
            //                 size,
            //                 &screen.palette_tree,
            //                 &screen.palette,
            //             ),
            //             Some(Dither {
            //                 algorithm: Ordered,
            //                 threshold,
            //                 threshold_map: X2_2,
            //             }) => dither_slice::<OrderedAlg, 4>(
            //                 pixels,
            //                 threshold,
            //                 size,
            //                 &screen.palette_tree,
            //                 &screen.palette,
            //             ),
            //             Some(Dither {
            //                 algorithm: Ordered,
            //                 threshold,
            //                 threshold_map: X4_4,
            //             }) => dither_slice::<OrderedAlg, 16>(
            //                 pixels,
            //                 threshold,
            //                 size,
            //                 &screen.palette_tree,
            //                 &screen.palette,
            //             ),
            //             Some(Dither {
            //                 algorithm: Ordered,
            //                 threshold,
            //                 threshold_map: X8_8,
            //             }) => dither_slice::<OrderedAlg, 64>(
            //                 pixels,
            //                 threshold,
            //                 size,
            //                 &screen.palette_tree,
            //                 &screen.palette,
            //             ),
            //             Some(Dither {
            //                 algorithm: Pattern,
            //                 threshold,
            //                 threshold_map: X2_2,
            //             }) => dither_slice::<PatternAlg, 4>(
            //                 pixels,
            //                 threshold,
            //                 size,
            //                 &screen.palette_tree,
            //                 &screen.palette,
            //             ),
            //             Some(Dither {
            //                 algorithm: Pattern,
            //                 threshold,
            //                 threshold_map: X4_4,
            //             }) => dither_slice::<PatternAlg, 16>(
            //                 pixels,
            //                 threshold,
            //                 size,
            //                 &screen.palette_tree,
            //                 &screen.palette,
            //             ),
            //             Some(Dither {
            //                 algorithm: Pattern,
            //                 threshold,
            //                 threshold_map: X8_8,
            //             }) => dither_slice::<PatternAlg, 64>(
            //                 pixels,
            //                 threshold,
            //                 size,
            //                 &screen.palette_tree,
            //                 &screen.palette,
            //             ),
            //         }
            //     });

            //     draw_spatial(
            //         &sprite,
            //         (),
            //         &mut layer_image,
            //         *position,
            //         *anchor,
            //         *canvas,
            //         None,
            //         filter.and_then(|filter| filters.get(filter)),
            //         camera,
            //     );
            // }

            for (sprite, position, anchor, canvas, frame, filter) in sprites {
                let Some(sprite) = sprite_assets.get(&**sprite) else {
                    continue;
                };

                draw_spatial(
                    sprite,
                    (),
                    &mut layer_slice,
                    position,
                    anchor,
                    canvas,
                    frame.copied(),
                    filter.and_then(|filter| filters.get(&**filter)),
                    camera,
                );
            }

            for (text, pos, alignment, canvas, frame, filter) in texts {
                let Some(typeface) = typefaces.get(&text.typeface) else {
                    continue;
                };

                let line_break_count = text.line_breaks.len() as u32;
                let mut size = uvec2(
                    0,
                    (line_break_count + 1) * typeface.height + line_break_count,
                );
                let mut x = 0;
                let mut y = 0;
                let mut chars = Vec::new();
                let mut line_break_index = 0;

                for (index, char) in text.value.chars().enumerate() {
                    if let Some(char) = typeface.characters.get(&char) {
                        if x != 0 {
                            x += 1;
                        }

                        chars.push((x, y, char));
                        x += char.data.size().x;

                        if x > size.x {
                            size.x = x;
                        }
                    } else if let Some(separator) = typeface.separators.get(&char) {
                        x += separator.width;
                    } else {
                        error!(r#"character "{char}" in text isn't in typeface"#);
                    }

                    if text.line_breaks.get(line_break_index).copied() == Some(index as u32) {
                        line_break_index += 1;
                        y += typeface.height + 1;
                        x = 0;
                    }
                }

                let top_left = *pos - alignment.pos(size).as_ivec2() + ivec2(0, size.y as i32 - 1);

                for (x, y, char) in chars {
                    draw_spatial(
                        char,
                        (),
                        &mut layer_slice,
                        PxPosition(top_left + ivec2(x as i32, -(y as i32))),
                        PxAnchor::TopLeft,
                        canvas,
                        frame.copied(),
                        filter.and_then(|filter| filters.get(&**filter)),
                        camera,
                    );
                }
            }

            for (rect, filter, pos, anchor, canvas, frame, invert) in clip_rects {
                if let Some(filter) = filters.get(&**filter) {
                    draw_spatial(
                        &(rect, filter),
                        invert,
                        &mut layer_slice,
                        pos,
                        anchor,
                        canvas,
                        frame.copied(),
                        empty(),
                        camera,
                    );
                }
            }

            // This is where I draw the line! /j
            #[cfg(feature = "line")]
            for (line, filter, canvas, frame, invert) in clip_lines {
                if let Some(filter) = filters.get(&**filter) {
                    draw_line(
                        line,
                        filter,
                        invert,
                        &mut layer_slice,
                        canvas,
                        frame.copied(),
                        camera,
                    );
                }
            }

            for (filter, frame) in clip_filters {
                if let Some(filter) = filters.get(&**filter) {
                    draw_filter(filter, frame.copied(), &mut layer_slice);
                }
            }

            image_slice.draw(&layer_image);

            for (rect, filter, pos, anchor, canvas, frame, invert) in over_rects {
                if let Some(filter) = filters.get(&**filter) {
                    draw_spatial(
                        &(rect, filter),
                        invert,
                        &mut image_slice,
                        pos,
                        anchor,
                        canvas,
                        frame.copied(),
                        empty(),
                        camera,
                    );
                }
            }

            #[cfg(feature = "line")]
            for (line, filter, canvas, frame, invert) in over_lines {
                if let Some(filter) = filters.get(&**filter) {
                    draw_line(
                        line,
                        filter,
                        invert,
                        &mut image_slice,
                        canvas,
                        frame.copied(),
                        camera,
                    );
                }
            }

            for (filter, frame) in over_filters {
                if let Some(filter) = filters.get(&**filter) {
                    draw_filter(filter, frame.copied(), &mut image_slice);
                }
            }
        }

        let cursor = world.resource::<CursorState>();

        if let PxCursor::Filter {
            idle,
            left_click,
            right_click,
        } = world.resource()
            && let Some(cursor_pos) = **world.resource::<PxCursorPosition>()
            && let Some(PxFilterAsset(filter)) = filters.get(match cursor {
                CursorState::Idle => idle,
                CursorState::Left => left_click,
                CursorState::Right => right_click,
            })
            && let mut image = PxImageSliceMut::from_image_mut(&mut image).unwrap()
            && let Some(pixel) = image.get_pixel_mut(IVec2::new(
                cursor_pos.x as i32,
                image.height() as i32 - 1 - cursor_pos.y as i32,
            ))
        {
            if let Some(new_pixel) = filter.get_pixel(IVec2::new(*pixel as i32, 0)) {
                *pixel = new_pixel;
            } else {
                error!("`PxCursor` filter is the wrong size");
            }
        }

        let Some(uniform_binding) = world.resource::<PxUniformBuffer>().binding() else {
            return Ok(());
        };

        let texture = render_context
            .render_device()
            .create_texture(&image.texture_descriptor);

        let Ok(pixel_size) = image.texture_descriptor.format.pixel_size() else {
            return Ok(());
        };

        world.resource::<RenderQueue>().write_texture(
            texture.as_image_copy(),
            image.data.as_ref().unwrap(),
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(image.width() * pixel_size as u32),
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
                depth_slice: None,
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
    palette: Res<PaletteHandle>,
    palettes: Res<Assets<Palette>>,
) {
    if !palette_handle.is_changed() && !*waiting_for_load {
        return;
    }

    let Some(palette) = palettes.get(&**palette) else {
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

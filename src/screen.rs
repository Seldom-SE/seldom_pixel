//! Screen and rendering

use std::{collections::BTreeMap, marker::PhantomData};

use bevy::{
    core_pipeline::core_2d::graph::{Core2d, Node2d},
    image::TextureFormatPixelInfo,
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
        view::ViewTarget,
        Render, RenderApp, RenderSet,
    },
    window::{PrimaryWindow, WindowResized},
};

#[cfg(feature = "line")]
use crate::line::{draw_line, LineComponents};
use crate::{
    animation::{copy_animation_params, draw_spatial, LastUpdate},
    cursor::{CursorState, PxCursorPosition},
    filter::{draw_filter, FilterComponents},
    image::{PxImage, PxImageSliceMut},
    map::{MapComponents, PxTile, TileComponents},
    palette::{PaletteHandle, PaletteParam},
    position::PxLayer,
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

fn insert_screen(size: ScreenSize) -> impl Fn(Query<&Window, With<PrimaryWindow>>, Commands) {
    move |windows, mut commands| {
        let window = windows.single();

        commands.insert_resource(Screen {
            size,
            computed_size: size.compute(Vec2::new(window.width(), window.height())),
            window_aspect_ratio: window.width() / window.height(),
            palette: [Vec3::ZERO; 256],
            // palette_tree: ImmutableKdTree::from(&[][..]),
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
        let mut layer_contents =
            BTreeMap::<_, (Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>, Vec<_>)>::default();
        #[cfg(not(feature = "line"))]
        let mut layer_contents =
            BTreeMap::<_, (Vec<_>, Vec<_>, Vec<_>, (), Vec<_>, (), Vec<_>)>::default();

        for (map, position, layer, canvas, animation, filter) in self.maps.iter_manual(world) {
            if let Some((maps, _, _, _, _, _, _)) = layer_contents.get_mut(layer) {
                maps.push((map, position, canvas, animation, filter));
            } else {
                layer_contents.insert(
                    layer.clone(),
                    (
                        vec![(map, position, canvas, animation, filter)],
                        // default(),
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

        for (sprite, position, anchor, layer, canvas, animation, filter) in
            self.sprites.iter_manual(world)
        {
            if let Some((_, sprites, _, _, _, _, _)) = layer_contents.get_mut(layer) {
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
                    ),
                );
            }
        }

        for (text, pos, alignment, layer, canvas, animation, filter) in
            self.texts.iter_manual(world)
        {
            if let Some((_, _, texts, _, _, _, _)) = layer_contents.get_mut(layer) {
                texts.push((text, pos, alignment, canvas, animation, filter));
            } else {
                layer_contents.insert(
                    layer.clone(),
                    (
                        default(),
                        default(),
                        vec![(text, pos, alignment, canvas, animation, filter)],
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
                                lines,
                                default(),
                            )
                        },
                    );
                }
            }
        }

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
                if let Some((_, _, _, _, clip_filters, _, over_filters)) =
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

        let mut layer_image = PxImage::<Option<u8>>::empty_from_image(&image);
        let mut image_slice = PxImageSliceMut::from_image_mut(&mut image);

        #[allow(unused_variables)]
        for (
            _,
            (
                maps,
                // image_to_sprites,
                sprites,
                texts,
                clip_lines,
                clip_filters,
                over_lines,
                over_filters,
            ),
        ) in layer_contents.into_iter()
        {
            layer_image.clear();

            for (map, position, canvas, animation, map_filter) in maps {
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

            for (sprite, position, anchor, canvas, animation, filter) in sprites {
                let Some(sprite) = sprite_assets.get(&**sprite) else {
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
                    filter.and_then(|filter| filters.get(&**filter)),
                    camera,
                );
            }

            for (text, pos, alignment, canvas, animation, filter) in texts {
                let Some(typeface) = typefaces.get(&text.typeface) else {
                    continue;
                };

                let mut line_y = text.computed_size.y - 1;
                let mut character_x = 0;
                let mut was_character = false;
                for character in text.value.chars() {
                    if typeface.separators.contains_key(&character) {
                        line_y -= typeface.height + 1;
                        character_x = 0;
                        was_character = false;
                    }

                    character_x += if let Some(character) = typeface.characters.get(&character) {
                        was_character = true;

                        draw_spatial(
                            character,
                            (),
                            &mut layer_image,
                            PxPosition(**pos + IVec2::new(character_x as i32, line_y as i32)),
                            *alignment,
                            *canvas,
                            copy_animation_params(animation, last_update),
                            filter.and_then(|filter| filters.get(&**filter)),
                            camera,
                        );

                        character.data.width() as u32 + 1
                    } else if let Some(separator) = typeface.separators.get(&character) {
                        if was_character {
                            character_x -= 1;
                        }
                        was_character = false;

                        separator.width
                    } else {
                        error!("received character '{character}' that isn't in typeface");
                        0
                    };
                }
            }

            // This is where I draw the line! /j
            #[cfg(feature = "line")]
            for (line, filter, canvas, animation) in clip_lines {
                if let Some(filter) = filters.get(&**filter) {
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
                if let Some(filter) = filters.get(&**filter) {
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
                if let Some(filter) = filters.get(&**filter) {
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
                if let Some(filter) = filters.get(&**filter) {
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
                if let Some(PxFilterAsset(filter)) = filters.get(match cursor {
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

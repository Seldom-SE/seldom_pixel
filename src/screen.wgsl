// struct VertexOutput {
//     @location(0) world_position: vec4<f32>,
//     @location(1) world_normal: vec3<f32>,
//     @location(2) uv: vec2<f32>,
// };

// struct ScreenMaterial {
//     palette: array<vec3<f32>, 256>,
// };

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var out_texture: texture_2d<f32>;
@group(0) @binding(1) var out_sampler: sampler;
@group(0) @binding(2) var in_texture: texture_2d<u32>;

@fragment
fn fragment(input: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>((textureLoad(in_texture, vec2(0, 0), 0))) / 255.0;
}

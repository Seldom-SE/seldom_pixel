struct VertexOutput {
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct ScreenMaterial {
    palette: array<vec3<f32>, 256>,
};

@group(2) @binding(0)
var<uniform> uniform_data: ScreenMaterial;

@group(2) @binding(1)
var texture: texture_2d<u32>;

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(uniform_data.palette[textureLoad(texture, vec2<i32>(
        vec2<f32>(textureDimensions(texture)) * input.uv
    ), 0).r], 1.);
}

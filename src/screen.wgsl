struct PxUniform {
    palette: array<vec3<f32>, 256>,
    fit_factor: vec2<f32>,
};

@group(0) @binding(0) var texture: texture_2d<u32>;
@group(0) @binding(1) var<uniform> uniform: PxUniform;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

var<private> VERTEX_U: array<f32, 6> = array(0., 0., 1., 0., 1., 1.);

@vertex fn vertex(@builtin(vertex_index) index: u32) -> VertexOut {
    let uv = vec2(VERTEX_U[index], f32(index & 1));
    return VertexOut(vec4((uv - 0.5) * vec2(2., -2.) * uniform.fit_factor, 0., 1.), uv);
}

@fragment fn fragment(vert: VertexOut) -> @location(0) vec4<f32> {
    return vec4(uniform.palette[
        textureLoad(texture, vec2<i32>(vec2<f32>(textureDimensions(texture)) * vert.uv), 0).r
    ], 1.);
}

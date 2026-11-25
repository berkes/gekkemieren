@group(0) @binding(0) var dashboard_texture: texture_2d<f32>;
@group(0) @binding(1) var dashboard_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

const FULLSCREEN_QUAD = array<vec2<f32>, 6>(
    vec2(-1.0,  1.0), vec2( 1.0,  1.0), vec2(-1.0, -1.0),
    vec2( 1.0,  1.0), vec2( 1.0, -1.0), vec2(-1.0, -1.0),
);

const UVS = array<vec2<f32>, 6>(
    vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0),
    vec2(1.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0),
);

@vertex
fn vs_main(@builtin(vertex_index) vertex: u32) -> VertexOutput {
    let pos = FULLSCREEN_QUAD[vertex];
    return VertexOutput(vec4<f32>(pos, 0.0, 1.0), UVS[vertex]);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(dashboard_texture, dashboard_sampler, in.uv);
}

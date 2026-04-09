struct Ant {
    position: vec2<f32>,
    direction: vec2<f32>,
    ant_type: u32,
    _pad: u32,
}

@group(0) @binding(0) var<storage, read> ants: array<Ant>;

const QUAD_CORNERS = array<vec2<f32>, 6>(
    vec2(-1.0, -1.0), vec2( 1.0, -1.0), vec2(-1.0,  1.0),
    vec2( 1.0, -1.0), vec2( 1.0,  1.0), vec2(-1.0,  1.0),
);

const DOT_RADIUS: f32 = 0.001;

@vertex
fn vs_main(
    @builtin(vertex_index) vertex: u32,
    @builtin(instance_index) instance: u32,
) -> @builtin(position) vec4<f32> {
    let ant = ants[instance];
    let corner = QUAD_CORNERS[vertex] * DOT_RADIUS;
    let pos = ant.position + corner;
    let clip = pos * 2.0 - vec2<f32>(1.0);
    return vec4<f32>(clip.x, -clip.y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

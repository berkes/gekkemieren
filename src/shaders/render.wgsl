struct Ant {
    position: vec2<f32>,
    direction: vec2<f32>,
    ant_type: u32,
    _pad: u32,
}

@group(0) @binding(0) var<storage, read> ants: array<Ant>;

const FORAGER_COLOR: vec4<f32> = vec4<f32>(0.227, 0.047, 0.639, 1.0); // #3a0ca3
const SCOUT_COLOR:   vec4<f32> = vec4<f32>(0.263, 0.380, 0.933, 1.0); // #4361ee

struct VertexOutput {
    @builtin(position)                    position: vec4<f32>,
    @location(0) @interpolate(flat) ant_type: u32,
}

@vertex
fn vs_main(@builtin(instance_index) instance: u32) -> VertexOutput {
    let ant = ants[instance];
    let clip = ant.position * 2.0 - vec2<f32>(1.0);
    return VertexOutput(vec4<f32>(clip.x, -clip.y, 0.0, 1.0), ant.ant_type);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return select(FORAGER_COLOR, SCOUT_COLOR, in.ant_type == 1u);
}

struct Ant {
    position: vec2<f32>,
    direction: vec2<f32>,
    ant_type: u32,
    _pad: u32,
}

@group(0) @binding(0) var<storage, read> ants: array<Ant>;

@vertex
fn vs_main(@builtin(instance_index) instance: u32) -> @builtin(position) vec4<f32> {
    let ant = ants[instance];
    let clip = ant.position * 2.0 - vec2<f32>(1.0);
    return vec4<f32>(clip.x, -clip.y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

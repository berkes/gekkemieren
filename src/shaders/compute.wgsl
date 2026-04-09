struct Ant {
    position: vec2<f32>,
    direction: vec2<f32>,
    ant_type: u32,
    _pad: u32,
}

@group(0) @binding(0) var<storage, read_write> ants: array<Ant>;

@compute
@workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if index >= arrayLength(&ants) {
        return;
    }

    var ant = ants[index];
    ant.position = clamp(ant.position + ant.direction, vec2<f32>(0.0), vec2<f32>(1.0));
    ants[index] = ant;
}

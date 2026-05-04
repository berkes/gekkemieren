struct Ant {
    position: vec2<f32>,
    direction: vec2<f32>,
    ant_type: u32,
    carries_food: u32,
}

struct ColorScheme {
    background: vec4<f32>,
    forager:    vec4<f32>,
    scout:      vec4<f32>,
    homing_pheromone: vec4<f32>,
    food_pheromone:  vec4<f32>,
    food:       vec4<f32>,
}

@group(0) @binding(0) var<storage, read> ants: array<Ant>;
@group(0) @binding(1) var<uniform> colors: ColorScheme;

struct VertexOutput {
    @builtin(position)              position: vec4<f32>,
    @location(0) @interpolate(flat) ant_type: u32,
    @location(1) @interpolate(flat) carries_food: u32,
}

@vertex
fn vs_main(@builtin(instance_index) instance: u32) -> VertexOutput {
    let ant = ants[instance];
    let clip = ant.position * 2.0 - vec2<f32>(1.0);
    return VertexOutput(vec4<f32>(clip.x, -clip.y, 0.0, 1.0), ant.ant_type, ant.carries_food);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // if in.carries_food == 1u {
    //     return colors.food;
    // }
    if in.ant_type == 1u {
        return colors.scout;
    }
    return colors.forager;
}

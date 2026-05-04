struct GridInfo {
    width: u32,
    height: u32,
    _pad1: u32,
    _pad2: u32,
}

struct GpuConfig {
    decay_amount: u32,
    max_strength: u32,
    deposit_amount: u32,
    dot_radius: f32,
    collision_radius: f32,
    collision_angle_min: f32,
    collision_angle_max: f32,
    forager_randomness: f32,
    scout_randomness: f32,
    sensor_distance: f32,
    sensor_angle: f32,
    n_ants: u32,
    base_speed: f32,
    scout_ratio: f32,
    ratio_step: f32,
    food_source_radius: f32,
    window_width: u32,
    window_height: u32,
}

struct ColorScheme {
    background: vec4<f32>,
    forager:    vec4<f32>,
    scout:      vec4<f32>,
    homing_pheromone: vec4<f32>,
    food_pheromone:  vec4<f32>,
    food:       vec4<f32>,
}

@group(0) @binding(0) var<storage, read> homing_pheromone_grid: array<u32>;
@group(0) @binding(1) var<storage, read> food_pheromone_grid: array<u32>;
@group(0) @binding(2) var<uniform> grid_info: GridInfo;
@group(0) @binding(3) var<uniform> config: GpuConfig;
@group(0) @binding(4) var<uniform> colors: ColorScheme;

const FULLSCREEN_QUAD = array<vec2<f32>, 6>(
    vec2(-1.0, -1.0), vec2( 1.0, -1.0), vec2(-1.0,  1.0),
    vec2( 1.0, -1.0), vec2( 1.0,  1.0), vec2(-1.0,  1.0),
);

@vertex
fn vs_main(@builtin(vertex_index) vertex: u32) -> @builtin(position) vec4<f32> {
    return vec4<f32>(FULLSCREEN_QUAD[vertex], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let x = min(u32(pos.x), grid_info.width - 1u);
    let y = min(u32(pos.y), grid_info.height - 1u);
    let idx = y * grid_info.width + x;

    // Sample both pheromone grids
    let homing_strength_u = min(homing_pheromone_grid[idx], config.max_strength);
    let food_strength_u = min(food_pheromone_grid[idx], config.max_strength);

    let homing_strength = f32(homing_strength_u) / f32(config.max_strength);
    let food_strength = f32(food_strength_u) / f32(config.max_strength);

    // Render only the strongest pheromone, set alpha to the strength.
    if homing_strength > food_strength {
        return vec4<f32>(colors.homing_pheromone.rgb, homing_strength);
    } else {
        return vec4<f32>(colors.food_pheromone.rgb, food_strength);
    }
}

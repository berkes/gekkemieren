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
    _pad1: u32,
    _pad2: u32,
}

struct ColorScheme {
    background: vec4<f32>,
    forager:    vec4<f32>,
    scout:      vec4<f32>,
    pheromone:  vec4<f32>,
}

@group(0) @binding(0) var<storage, read> pheromone_grid: array<u32>;
@group(0) @binding(1) var<uniform> grid_info: GridInfo;
@group(0) @binding(2) var<uniform> config: GpuConfig;
@group(0) @binding(3) var<uniform> colors: ColorScheme;

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
    let strength_u = min(pheromone_grid[y * grid_info.width + x], config.max_strength);
    let strength = f32(strength_u) / f32(config.max_strength);
    let color = mix(colors.background.rgb, colors.pheromone.rgb, strength);
    return vec4<f32>(color, 1.0);
}

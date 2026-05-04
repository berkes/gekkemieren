// HUD Shader - Renders a scout_ratio bar in the bottom right corner

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

@group(0) @binding(0) var<uniform> grid_info: GridInfo;
@group(0) @binding(1) var<uniform> config: GpuConfig;
@group(0) @binding(2) var<uniform> colors: ColorScheme;

// Bar configuration
const BAR_HEIGHT: f32 = 100.0;
const BAR_MARGIN: f32 = 10.0;
const BAR_WIDTH: f32 = 10.0;

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
    // In WebGPU, @builtin(position) in fragment shader gives pixel coordinates
    // with (0,0) at top-left, matching the render target dimensions
    let px = pos.x;
    let py = pos.y;

    let screen_width = f32(grid_info.width);
    let screen_height = f32(grid_info.height);

    // Bar area in bottom right corner (y increases downward from top)
    let bar_right = screen_width - BAR_MARGIN;
    let bar_left = bar_right - BAR_WIDTH;
    let bar_top = screen_height - BAR_MARGIN;
    let bar_bottom = bar_top - BAR_HEIGHT;

    // Check if pixel is inside the bar
    if px >= bar_left && px < bar_right && py >= bar_bottom && py < bar_top {
        // Calculate position within bar (0 to 1 from top to bottom)
        let bar_normalized_pos = (bar_top - py) / BAR_HEIGHT;

        // Use scout_ratio to determine color
        // If scout_ratio is 1, entire bar is scout color
        // If scout_ratio is 0, entire bar is forager color
        // If scout_ratio is 0.2, top 20% is scout, rest is forager
        if bar_normalized_pos < config.scout_ratio {
            return colors.scout;
        } else {
            return colors.forager;
        }
    }

    // Transparent outside the bar
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}

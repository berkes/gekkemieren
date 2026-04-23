struct GridInfo {
    width: u32,
    height: u32,
    _pad1: u32,
    _pad2: u32,
}

struct ColorScheme {
    background: vec4<f32>,
    forager:    vec4<f32>,
    scout:      vec4<f32>,
    pheromone:  vec4<f32>,
    food:       vec4<f32>,
}

@group(0) @binding(0) var<storage, read> food_grid: array<u32>;
@group(0) @binding(1) var<uniform> grid_info: GridInfo;
@group(0) @binding(2) var<uniform> colors: ColorScheme;

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
    let food_value = food_grid[y * grid_info.width + x];
    
    // If there's food (value = 1), display it with the food color
    // Otherwise, display background (transparent or mixed)
    if food_value > 0u {
        return colors.food;
    } else {
        // Return transparent - food is only shown where present
        // This allows pheromones to show through
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}

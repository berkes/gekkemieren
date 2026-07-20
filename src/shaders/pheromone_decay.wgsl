struct GpuConfig {
    decay_ratio: f32,
    deposit_ratio: f32,
    forager_randomness: f32,
    scout_randomness: f32,
    sensor_distance: f32,
    sensor_angle: f32,
    scout_ratio: f32,
    _pad0: u32,
    food_source_center: vec2<f32>,
    food_source_radius: f32,
    _pad1: u32,
    colony_center: vec2<f32>,
    colony_radius: f32,
    _pad2: u32,
    _pad3: u32,
    _pad4: u32,
}

struct GridInfo {
    width: u32,
    height: u32,
    _pad1: u32,
    _pad2: u32,
}

@group(0) @binding(0) var<storage, read_write> homing_pheromone_grid: array<f32>;
@group(0) @binding(1) var<storage, read_write> food_pheromone_grid: array<f32>;
@group(0) @binding(2) var<uniform> config: GpuConfig;
@group(0) @binding(3) var<uniform> grid_info: GridInfo;

@compute
@workgroup_size(64)
fn pheromone_decay_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;

    // let old_val = homing_pheromone_grid[idx];
    // let new_val = clamp(
    //         old_val * (1.0 - config.decay_ratio),
    //         0.0,
    //         1.0
    //     );
    // let difference = old_val - new_val;
    // homing_pheromone_grid[idx] = new_val;

    // if idx + 1 < arrayLength(&homing_pheromone_grid) {
    //   homing_pheromone_grid[idx + 1] += difference * 0.24;
    // }
    // if idx - 1 >= 0 {
    //   homing_pheromone_grid[idx - 1] += difference * 0.24;
    // }
    // if (idx + grid_info.width) <= arrayLength(&homing_pheromone_grid) {
    //   homing_pheromone_grid[idx + grid_info.width] += difference * 0.24;
    // }
    // if (idx - grid_info.width) >= 0 {
    //   homing_pheromone_grid[idx - grid_info.width] += difference * 0.24;
    // }

    // food_pheromone_grid[idx] = clamp(
    //     food_pheromone_grid[idx] * (1.0 - config.decay_ratio),
    //     0.0,
    //     1.0
    // );
    diffuse_homing(idx);
    diffuse_food(idx);
}

fn in_bounds(idx: u32, length: u32) -> bool {
    return idx < length && idx >= 0;
}

fn diffuse_homing(idx: u32) {
    let length = arrayLength(&homing_pheromone_grid);
    if !in_bounds(idx, length) {
        return;
    }

    let old_val = homing_pheromone_grid[idx];
    let new_val = clamp(old_val * (1.0 - config.decay_ratio), 0.0, 1.0);
    let spread_val = (old_val - new_val) * (0.25 - (config.decay_ratio / 4.0));

    let top = idx + grid_info.width;
    let bottom = idx - grid_info.width;
    let left = idx - 1;
    let right = idx + 1;


    if in_bounds(top, length) {
        homing_pheromone_grid[top] += spread_val;
    }
    if in_bounds(bottom, length) {
        homing_pheromone_grid[bottom] += spread_val;
    }
    if in_bounds(left, length) {
        homing_pheromone_grid[left] += spread_val;
    }
    if in_bounds(right, length) {
        homing_pheromone_grid[right] += spread_val;
    }
    homing_pheromone_grid[idx] = new_val;
}

fn diffuse_food(idx: u32) {
    let length = arrayLength(&food_pheromone_grid);
    if !in_bounds(idx, length) {
        return;
    }

    let old_val = food_pheromone_grid[idx];
    let new_val = clamp(old_val * (1.0 - config.decay_ratio), 0.0, 1.0);
    let spread_val = (old_val - new_val) * (0.25 - (config.decay_ratio / 4.0));

    let top = idx + grid_info.width;
    let bottom = idx - grid_info.width;
    let left = idx - 1;
    let right = idx + 1;

    if in_bounds(top, length) {
        food_pheromone_grid[top] += spread_val;
    }
    if in_bounds(bottom, length) {
        food_pheromone_grid[bottom] += spread_val;
    }
    if in_bounds(left, length) {
        food_pheromone_grid[left] += spread_val;
    }
    if in_bounds(right, length) {
        food_pheromone_grid[right] += spread_val;
    }
    food_pheromone_grid[idx] = new_val;
}

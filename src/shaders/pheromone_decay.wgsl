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
    diffusion_ratio: f32,
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

@group(0) @binding(0) var<storage, read> homing_pheromone_grid: array<f32>;
@group(0) @binding(1) var<storage, read_write> homing_pheromone_grid_next: array<f32>;
@group(0) @binding(2) var<storage, read> food_pheromone_grid: array<f32>;
@group(0) @binding(3) var<storage, read_write> food_pheromone_grid_next: array<f32>;
@group(0) @binding(4) var<uniform> config: GpuConfig;
@group(0) @binding(5) var<uniform> grid_info: GridInfo;

@compute
@workgroup_size(64)
fn pheromone_decay_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    diffuse_homing(idx);
    diffuse_food(idx);
    // decay_food(idx);
    // decay_homing(idx);
}

fn decayed(v: f32) -> f32 {
    return clamp(v * (1.0 - config.decay_ratio), 0.0, 1.0);
}

fn decay_food(idx: u32) {
    if (grid_info.width == 0 || grid_info.height == 0) {
        return;
    }
    food_pheromone_grid_next[idx] = clamp(food_pheromone_grid[idx] * (1.0 - config.decay_ratio), 0.0, 1.0);
}

fn decay_homing(idx: u32) {
    if (grid_info.width == 0 || grid_info.height == 0) {
        return;
    }
    homing_pheromone_grid_next[idx] = clamp(homing_pheromone_grid[idx] * (1.0 - config.decay_ratio), 0.0, 1.0);
}

// Gather-style diffusion: every invocation reads its neighbours from the read
// grid and writes only its own cell in the write grid, so each cell has
// exactly one writer and the pass is free of write races.
// Diffusion moves pheromone between cells; decay then removes a fixed
// fraction of the total every tick, so pheromone cannot accumulate forever.
fn diffuse_homing(idx: u32) {
    let length = arrayLength(&homing_pheromone_grid);
    if idx >= length {
        return;
    }

    let width = grid_info.width;
    let x = idx % width;
    let k = config.diffusion_ratio;

    var neighbor_sum: f32 = 0.0;
    if idx >= width {
        neighbor_sum += homing_pheromone_grid[idx - width];
    }
    if idx + width < length {
        neighbor_sum += homing_pheromone_grid[idx + width];
    }
    if x > 0u {
        neighbor_sum += homing_pheromone_grid[idx - 1u];
    }
    if x < width - 1u {
        neighbor_sum += homing_pheromone_grid[idx + 1u];
    }

    let diffused = (1.0 - 4.0 * k) * homing_pheromone_grid[idx] + k * neighbor_sum;
    homing_pheromone_grid_next[idx] = clamp(diffused * (1.0 - config.decay_ratio), 0.0, 1.0);
}

fn diffuse_food(idx: u32) {
    let length = arrayLength(&food_pheromone_grid);
    if idx >= length {
        return;
    }

    let width = grid_info.width;
    let x = idx % width;
    let k = config.diffusion_ratio;

    var neighbor_sum: f32 = 0.0;
    if idx >= width {
        neighbor_sum += food_pheromone_grid[idx - width];
    }
    if idx + width < length {
        neighbor_sum += food_pheromone_grid[idx + width];
    }
    if x > 0u {
        neighbor_sum += food_pheromone_grid[idx - 1u];
    }
    if x < width - 1u {
        neighbor_sum += food_pheromone_grid[idx + 1u];
    }

    let diffused = (1.0 - 4.0 * k) * food_pheromone_grid[idx] + k * neighbor_sum;
    food_pheromone_grid_next[idx] = clamp(diffused * (1.0 - config.decay_ratio), 0.0, 1.0);
}

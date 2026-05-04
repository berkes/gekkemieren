struct GpuConfig {
    decay_amount: u32,
    max_strength: u32,
    deposit_amount: u32,
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

@group(0) @binding(0) var<storage, read_write> homing_pheromone_grid: array<atomic<u32>>;
@group(0) @binding(1) var<storage, read_write> food_pheromone_grid: array<atomic<u32>>;
@group(0) @binding(2) var<uniform> config: GpuConfig;

@compute
@workgroup_size(64)
fn pheromone_decay_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if idx >= arrayLength(&homing_pheromone_grid) {
        return;
    }

    // Decay homing pheromone grid
    let homing_current = min(atomicLoad(&homing_pheromone_grid[idx]), config.max_strength);
    if homing_current < config.decay_amount {
        atomicStore(&homing_pheromone_grid[idx], 0u);
    } else {
        atomicStore(&homing_pheromone_grid[idx], homing_current - config.decay_amount);
    }

    // Decay food pheromone grid
    let food_current = min(atomicLoad(&food_pheromone_grid[idx]), config.max_strength);
    if food_current < config.decay_amount {
        atomicStore(&food_pheromone_grid[idx], 0u);
    } else {
        atomicStore(&food_pheromone_grid[idx], food_current - config.decay_amount);
    }
}

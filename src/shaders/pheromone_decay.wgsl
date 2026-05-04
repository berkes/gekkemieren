struct GpuConfig {
    decay_ratio: f32,
    deposit_ratio: f32,
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

@group(0) @binding(0) var<storage, read_write> homing_pheromone_grid: array<f32>;
@group(0) @binding(1) var<storage, read_write> food_pheromone_grid: array<f32>;
@group(0) @binding(2) var<uniform> config: GpuConfig;

@compute
@workgroup_size(64)
fn pheromone_decay_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if idx >= arrayLength(&homing_pheromone_grid) {
        return;
    }

    homing_pheromone_grid[idx] = clamp(
        homing_pheromone_grid[idx] * (1.0 - config.decay_ratio),
        0.0,
        1.0
    );

    food_pheromone_grid[idx] = clamp(
        food_pheromone_grid[idx] * (1.0 - config.decay_ratio),
        0.0,
        1.0
    );
}

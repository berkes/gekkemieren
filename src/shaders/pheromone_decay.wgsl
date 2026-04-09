struct SimConfig {
    decay_amount: u32,
    max_strength: u32,
    deposit_amount: u32,
    _pad: u32,
}

@group(0) @binding(0) var<storage, read_write> pheromone_grid: array<atomic<u32>>;
@group(0) @binding(1) var<uniform> config: SimConfig;

@compute
@workgroup_size(64)
fn pheromone_decay_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    if idx >= arrayLength(&pheromone_grid) {
        return;
    }
    let current = min(atomicLoad(&pheromone_grid[idx]), config.max_strength);
    if current < config.decay_amount {
        atomicStore(&pheromone_grid[idx], 0u);
    } else {
        atomicStore(&pheromone_grid[idx], current - config.decay_amount);
    }
}

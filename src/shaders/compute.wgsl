struct Ant {
    position: vec2<f32>,
    direction: vec2<f32>,
    ant_type: u32,
    carries_food: u32,
}

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
    deposit_amount_carrying_food: u32,
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
    homing_strength: f32,
    window_width: u32,
    window_height: u32,
    _pad1: u32,
    _pad2: u32,
}

@group(0) @binding(0) var<storage, read_write> ants: array<Ant>;
@group(0) @binding(2) var<storage, read_write> pheromone_grid: array<atomic<u32>>;
@group(0) @binding(3) var<uniform> grid_info: GridInfo;
@group(0) @binding(4) var<uniform> config: GpuConfig;
@group(0) @binding(5) var<storage, read_write> food_grid: array<atomic<u32>>;

// Hash constants - primes used for pseudo-random number generation
const HASH_COORD_SCALE: vec2<f32> = vec2<f32>(127.1, 311.7);
const HASH_FINAL_SCALE: f32 = 43758.5453;
const INDEX_OFFSET_SCALE: f32 = 0.1;

// Generates pseudo-random value in [min, max) range.
// seed.xy = position, seed.z = index (as float)
// Uses symmetric index offset to ensure no directional bias.
fn random(seed: vec3<f32>, min: f32, max: f32) -> f32 {
    let pos = seed.xy;
    let index = seed.z;
    let index_offset = vec2<f32>(index);
    let seed_pos = pos + index_offset * INDEX_OFFSET_SCALE;

    // Mix coordinates by scaling with prime constants and wrapping
    let mixed_coords = fract(seed_pos * HASH_COORD_SCALE);

    // Combine components and wrap again to produce final [0, 1) value
    let hash_value = fract((mixed_coords.x + mixed_coords.y) * HASH_FINAL_SCALE);

    // Scale from [0, 1) to [min, max)
    return min + hash_value * (max - min);
}

fn random_collision_angle(pos: vec2<f32>) -> f32 {
    return random(vec3<f32>(pos, 1.0), config.collision_angle_min, config.collision_angle_max);
}

struct Colony {
    center: vec2<f32>,
    half_size: f32,
    _pad: f32,
}

@group(0) @binding(1) var<uniform> colony: Colony;

fn in_colony(pos: vec2<f32>) -> bool {
    let d = abs(pos - colony.center);
    return d.x < colony.half_size && d.y < colony.half_size;
}

fn sample_pheromone_area(pos: vec2<f32>) -> f32 {
    let clamped = clamp(pos, vec2<f32>(0.0), vec2<f32>(1.0));
    let center_x = clamp(u32(clamped.x * f32(grid_info.width)), 0u, grid_info.width - 1u);
    let center_y = clamp(u32(clamped.y * f32(grid_info.height)), 0u, grid_info.height - 1u);

    var total: f32 = 0.0;
    var count: f32 = 0.0;

    for (var dy: i32 = -1; dy <= 1; dy++) {
        for (var dx: i32 = -1; dx <= 1; dx++) {
            let sx = i32(center_x) + dx;
            let sy = i32(center_y) + dy;

            // Clamp to grid bounds
            if sx >= 0 && sx < i32(grid_info.width) && sy >= 0 && sy < i32(grid_info.height) {
                let idx = u32(sy) * grid_info.width + u32(sx);
                total += f32(atomicLoad(&pheromone_grid[idx]));
                count += 1.0;
            }
        }
    }
    return total / count;
}

fn rotate(v: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec2<f32>(c * v.x - s * v.y, s * v.x + c * v.y);
}

/// Compute steering direction for ants carrying food to head toward colony.
/// Returns the direction with homing angle applied, or the original direction if in colony.
fn apply_homing(direction: vec2<f32>, position: vec2<f32>, strength: f32) -> vec2<f32> {
    if in_colony(position) {
        return direction;
    }
    let to_colony = colony.center - position;
    let len_sq = dot(to_colony, to_colony);
    if (len_sq <= 0.000001) {
        return direction;
    }
    let current_angle = atan2(direction.y, direction.x);
    let target_angle = atan2(to_colony.y, to_colony.x);
    let angle_diff = target_angle - current_angle;
    // Normalize angle difference to [-pi, pi] range
    let pi = 3.141592653589793;
    let angle_diff_norm = angle_diff - 2.0 * pi * floor((angle_diff + pi) / (2.0 * pi));
    return rotate(direction, angle_diff_norm * strength);
}

@compute
@workgroup_size(64)
fn collision_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    let count = arrayLength(&ants);
    if index >= count {
        return;
    }

    let ant = ants[index];
    let pos = ant.position;

    if in_colony(pos) {
        return;
    }

    for (var i: u32 = 0u; i < count; i++) {
        if i == index { continue; }
        if distance(pos, ants[i].position) < config.collision_radius {
            ants[index].direction = rotate(ant.direction, random_collision_angle(pos));
            break;
        }
    }
}

@compute
@workgroup_size(64)
fn movement_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if index >= arrayLength(&ants) {
        return;
    }

    var ant = ants[index];
    let next = ant.position + ant.direction;
    if next.x < 0.0 || next.x > 1.0 { ant.direction.x = -ant.direction.x; }
    if next.y < 0.0 || next.y > 1.0 { ant.direction.y = -ant.direction.y; }

    ant.position = clamp(next, vec2<f32>(0.0), vec2<f32>(1.0));

    // Food drop logic: drop food when entering colony
    if ant.carries_food == 1u && in_colony(ant.position) {
        ant.carries_food = 0u;
    }

    // If ant carries food, apply homing toward colony
    if ant.carries_food == 1u {
        ant.direction = apply_homing(ant.direction, ant.position, config.homing_strength);
    }

    // Pheromone following for foragers and ants carrying food
    if ant.ant_type == 0u || ant.carries_food == 1u {
        let dir_norm = normalize(ant.direction);
        let left_pos = ant.position + rotate(dir_norm, config.sensor_angle) * config.sensor_distance;
        let right_pos = ant.position + rotate(dir_norm, -config.sensor_angle) * config.sensor_distance;
        let left_sample = sample_pheromone_area(left_pos);
        let right_sample = sample_pheromone_area(right_pos);

        if left_sample > right_sample {
            ant.direction = rotate(ant.direction, config.sensor_angle);
        } else if right_sample > left_sample {
            ant.direction = rotate(ant.direction, -config.sensor_angle);
        } else if left_sample == 0.0 && right_sample == 0.0 && ant.ant_type == 0u {
            // followers bactrack, they never venture into unknown
            // ant.direction = apply_homing(ant.direction, ant.position, 0.05);
        }
    }

    // Apply random direction change based on ant type
    let randomness = select(config.forager_randomness, config.scout_randomness, ant.ant_type == 1u);
    ant.direction = rotate(ant.direction, random(vec3<f32>(ant.position, f32(index)), -1.0, 1.0) * randomness);

    // Food pickup logic: only pick up food if not already carrying
    if ant.carries_food == 0u {
        let cell_x = clamp(u32(ant.position.x * f32(grid_info.width)), 0u, grid_info.width - 1u);
        let cell_y = clamp(u32(ant.position.y * f32(grid_info.height)), 0u, grid_info.height - 1u);
        let food_idx = cell_y * grid_info.width + cell_x;

        // Try to pick up food - use atomic exchange to get and set in one operation
        // If the food cell was 1, we pick it up (set to 0) and the ant now carries food
        let prev_food = atomicExchange(&food_grid[food_idx], 0u);
        if prev_food == 1u {
            ant.carries_food = 1u;
        }
    }

    ants[index] = ant;

    let cell_x = clamp(u32(ant.position.x * f32(grid_info.width)), 0u, grid_info.width - 1u);
    let cell_y = clamp(u32(ant.position.y * f32(grid_info.height)), 0u, grid_info.height - 1u);
    let deposit = select(config.deposit_amount, config.deposit_amount_carrying_food, ant.carries_food == 1u);
    atomicAdd(&pheromone_grid[cell_y * grid_info.width + cell_x], deposit);
}

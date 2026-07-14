struct Ant {
    position: vec2<f32>,
    direction: vec2<f32>,
    ant_type: u32, // Forager = 0, Scout = 1
    carries_food: u32,
    lifetime: u32,
}

struct GridInfo {
    width: u32,
    height: u32,
    _pad1: u32,
    _pad2: u32,
}

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

@group(0) @binding(0) var<storage, read_write> ants: array<Ant>;
@group(0) @binding(1) var<storage, read_write> homing_pheromone_grid: array<f32>;
@group(0) @binding(2) var<storage, read_write> food_pheromone_grid: array<f32>;
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

fn in_colony(pos: vec2<f32>) -> bool {
    let dist = distance(pos, config.colony_center);
    return dist < config.colony_radius;
}

// Sample pheromone strength from a 3x3 area around pos from the specified grid.
// grid_selector: 0 = food_pheromone_grid, 1 = homing_pheromone_grid
fn sample_pheromone_area(pos: vec2<f32>, grid_selector: u32) -> f32 {
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
                // Select which grid to sample from based on grid_selector
                let strength = select(
                    f32(food_pheromone_grid[idx]),
                    f32(homing_pheromone_grid[idx]),
                    grid_selector == 1u
                );
                total += strength;
            }
            count += 1.0;
        }
    }
    return total / count;
}

fn rotate(v: vec2<f32>, angle: f32) -> vec2<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec2<f32>(c * v.x - s * v.y, s * v.x + c * v.y);
}

fn get_index(position: vec2<f32>) -> u32 {
    let cell_x = clamp(u32(position.x * f32(grid_info.width)), 0u, grid_info.width - 1u);
    let cell_y = clamp(u32(position.y * f32(grid_info.height)), 0u, grid_info.height - 1u);
    return cell_y * grid_info.width + cell_x;
}

fn deposit_pheromone_pixel(cell_x: u32, cell_y: u32, carries_food: u32) {
    let idx = cell_y * grid_info.width + cell_x;
    if carries_food == 1u {
        food_pheromone_grid[idx] = clamp(
            food_pheromone_grid[idx] + config.deposit_ratio,
            0.0,
            1.0
        );
    } else {
        homing_pheromone_grid[idx] = clamp(
            homing_pheromone_grid[idx] + config.deposit_ratio,
            0.0,
            1.0
        );
    }
}

// Maximum lifetime in ticks before an ant "dies" and respawns at colony
const MAX_LIFETIME: u32 = 500u;

@compute
@workgroup_size(64)
fn movement_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
    if index >= arrayLength(&ants) {
        return;
    }

    var ant = ants[index];
    var speed = select(1.0, 0.7, ant.carries_food == 1u);

    // Increment lifetime and check for death
    ant.lifetime += 1u;

    if ant.carries_food == 0u &&f32(ant.lifetime) >= (f32(MAX_LIFETIME) * random(vec3<f32>(ant.position, f32(index)), 0.6, 1.4)) {
        // Reset ant: teleport to colony, drop food, keep direction and type
        ant.position = config.colony_center;
        ant.carries_food = 0u;
        ant.lifetime = 0u;
    }

    if ant.carries_food == 1u && in_colony(ant.position) {
        ant.carries_food = 0u;
        ant.direction = -ant.direction;
    }

    // Reset lifetime whenever any ant enters the colony
    if in_colony(ant.position) {
        ant.lifetime = 0u;
    }

    // Pheromone following: ants without food follow food pheromones (grid 0),
    // ants with food follow homing pheromones (grid 1)
    let dir_norm = normalize(ant.direction);
    let left_pos = ant.position + rotate(dir_norm, config.sensor_angle) * config.sensor_distance;
    let right_pos = ant.position + rotate(dir_norm, -config.sensor_angle) * config.sensor_distance;

    var moved = false;
    // Check for food if ant is not carrying food
    if ant.carries_food == 0u {
        let left_food = atomicLoad(&food_grid[get_index(left_pos)]);
        let right_food = atomicLoad(&food_grid[get_index(right_pos)]);

        if left_food == 1u || right_food == 1u {
            if left_food > right_food {
                ant.direction = rotate(ant.direction, config.sensor_angle);
                moved = true;
            } else if right_food > left_food {
                ant.direction = rotate(ant.direction, -config.sensor_angle);
                moved = true;
            }
        }
    }

    // Select which pheromone grid to sample based on whether ant carries food
    if (!moved) {
        let grid_selector = ant.carries_food;
        let left_sample = sample_pheromone_area(left_pos, grid_selector);
        let right_sample = sample_pheromone_area(right_pos, grid_selector);

        if left_sample > right_sample {
            ant.direction = rotate(ant.direction, config.sensor_angle);
        } else if right_sample > left_sample {
            ant.direction = rotate(ant.direction, -config.sensor_angle);
        } else if ant.ant_type == 0u && right_sample <= 0.001 && left_sample <= 0.001 {
            // ant.direction = -ant.direction;
            speed = 0.001;
        }
    }

    // Apply random direction change based on ant type
    let randomness = select(config.forager_randomness, config.scout_randomness, ant.ant_type == 1u);
    ant.direction = rotate(ant.direction, random(vec3<f32>(ant.position, f32(index)), -1.0, 1.0) * randomness);

    // Food pickup logic: only pick up food if not already carrying
    if ant.carries_food == 0u {
        let food_idx = get_index(ant.position);

        // Try to pick up food - use atomic exchange to get and set in one operation
        // If the food cell was 1, we pick it up (set to 0) and the ant now carries food
        let prev_food = atomicExchange(&food_grid[food_idx], 0u);
        if prev_food == 1u {
            ant.carries_food = 1u;
            ant.direction = -ant.direction;
        }
    }

    let next = ant.position + (ant.direction * speed);
    if next.x < 0.0 || next.x > 1.0 { ant.direction.x = -ant.direction.x; }
    if next.y < 0.0 || next.y > 1.0 { ant.direction.y = -ant.direction.y; }

    ant.position = clamp(next, vec2<f32>(0.0), vec2<f32>(1.0));
    ants[index] = ant;

    let cell_x = clamp(u32(ant.position.x * f32(grid_info.width)), 0u, grid_info.width - 1u);
    let cell_y = clamp(u32(ant.position.y * f32(grid_info.height)), 0u, grid_info.height - 1u);

    deposit_pheromone_pixel(cell_x, cell_y, ant.carries_food);
}

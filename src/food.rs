//! Food grid module for the ant simulation.
//!
//! Food is represented as a boolean grid where each pixel can have food (1) or not (0).
//! This is similar to the pheromone grid but simpler - just presence/absence.

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridInfo {
    pub width: u32,
    pub height: u32,
    pub _pad: [u32; 2],
}

/// Represents a 2D grid of food.
/// Each cell is a u32 where 0 = no food, 1 = food present.
/// This matches the pheromone grid pattern for simplicity.
#[derive(Debug)]
pub struct FoodGrid {
    pub data: Vec<u32>,
    pub width: u32,
    pub height: u32,
}

pub struct FoodCircle {
    pub center: [f32; 2],
    pub radius: f32,
}

impl FoodGrid {
    /// Create a new empty food grid.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            data: vec![0u32; (width * height) as usize],
            width,
            height,
        }
    }

    /// Get food value at a position.
    pub fn get(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize]
        } else {
            0
        }
    }

    /// Set food value at a position (0 = no food, 1 = food).
    pub fn set(&mut self, x: u32, y: u32, value: u32) {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize] = value.clamp(0, 1);
        }
    }

    pub fn food_count(&self) -> usize {
        self.data.iter().sum::<u32>() as usize
    }

    /// Fill a circle with food.
    /// Center is in normalized coordinates [0, 1], radius is in normalized coordinates.
    pub fn fill_circle(&mut self, food: FoodCircle) {
        let center = food.center;
        let radius = food.radius;

        let cx = (center[0] * self.width as f32) as i32;
        let cy = (center[1] * self.height as f32) as i32;
        let r_squared = (radius * self.width as f32).powi(2) as i32;

        let r = (radius * self.width as f32) as i32;

        // Clamp bounds
        let min_x = (cx - r).max(0);
        let max_x = (cx + r).min(self.width as i32 - 1);
        let min_y = (cy - r).max(0);
        let max_y = (cy + r).min(self.height as i32 - 1);

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let dx = x - cx;
                let dy = y - cy;
                if dx * dx + dy * dy <= r_squared {
                    self.data[(y as u32 * self.width + x as u32) as usize] = 1;
                }
            }
        }
    }

    /// Get the grid info for GPU buffer.
    pub fn grid_info(&self) -> GridInfo {
        GridInfo {
            width: self.width,
            height: self.height,
            _pad: [0; 2],
        }
    }
}

/// Food spawner that creates initial food sources.
/// Spawns circles of food near random edges at regular intervals.
#[derive(Debug)]
pub struct FoodSpawner {
    pub food_grid: FoodGrid,
    pub radius: f32,
    pub spawn_interval: std::time::Duration,
    last_spawn_time: std::time::Instant,
    food_changed: bool,
}

impl FoodSpawner {
    /// Create a new food spawner with an empty grid.
    pub fn new(width: u32, height: u32, radius: f32) -> Self {
        Self {
            food_grid: FoodGrid::new(width, height),
            radius,
            spawn_interval: std::time::Duration::from_secs(300),
            last_spawn_time: std::time::Instant::now(),
            food_changed: true,
        }
    }

    /// Create a new food spawner with custom settings.
    pub fn with_settings(
        width: u32,
        height: u32,
        radius: f32,
        spawn_interval: std::time::Duration,
    ) -> Self {
        Self {
            food_grid: FoodGrid::new(width, height),
            radius,
            spawn_interval,
            last_spawn_time: std::time::Instant::now(),
            food_changed: true,
        }
    }

    pub fn spawn_food_circle(&mut self, center: [f32; 2]) {
        let food = FoodCircle {
            center,
            radius: self.radius,
        };
        self.food_grid.fill_circle(food);
        self.food_changed = true;
    }

    /// Spawn food as a circle near a random edge.
    /// The circle is placed randomly near one of the four edges.
    pub fn random_food_circle(&mut self, radius: f32) -> FoodCircle {
        use rand::RngExt;
        let mut rng = rand::rng();

        // Pick a random edge: 0 = top, 1 = right, 2 = bottom, 3 = left
        let edge: u32 = (rng.random::<f32>() * 4.0).floor() as u32;
        let offset = rng.random::<f32>() * 0.4 - 0.05;

        let center = match edge {
            0 => {
                // Top edge
                let x: f32 = rng.random::<f32>();
                [x, radius + offset]
            }
            1 => {
                // Right edge
                let y: f32 = rng.random::<f32>();
                [1.0 - radius - offset, y]
            }
            2 => {
                // Bottom edge
                let x: f32 = rng.random::<f32>();
                [x, 1.0 - radius - offset]
            }
            3 => {
                // Left edge
                let y: f32 = rng.random::<f32>();
                [radius + offset, y]
            }
            _ => [0.5, 0.5],
        };
        FoodCircle { center, radius }
    }

    /// Check if we are almost out of food and if so, spawn new
    /// Returns true if a new food source was spawned.
    pub fn try_spawn(&mut self) -> bool {
        if self.food_count() > 10 {
            return false;
        }

        let food_circle = self.random_food_circle(self.radius);
        self.spawn_food_circle(food_circle.center);
        self.last_spawn_time = std::time::Instant::now();
        true
    }

    /// Check if food grid has changed and reset the flag.
    pub fn food_changed(&mut self) -> bool {
        let changed = self.food_changed;
        self.food_changed = false;
        changed
    }

    pub(crate) fn food_count(&self) -> usize {
        self.food_grid.food_count()
    }

    /// Update the CPU-side food grid from GPU data.
    /// This syncs the CPU copy with changes made by ants on the GPU.
    pub fn update_from_gpu(&mut self, gpu_data: &[u32]) {
        self.food_grid.data.copy_from_slice(gpu_data);
    }
}

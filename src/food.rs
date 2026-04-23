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

    /// Fill a circle with food.
    /// Center is in normalized coordinates [0, 1], radius is in normalized coordinates.
    pub fn fill_circle(&mut self, center: [f32; 2], radius: f32) {
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
/// For now, spawns a circle of food near a random edge.
#[derive(Debug)]
pub struct FoodSpawner {
    pub food_grid: FoodGrid,
}

impl FoodSpawner {
    /// Create a new food spawner with an empty grid.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            food_grid: FoodGrid::new(width, height),
        }
    }

    /// Spawn food as a circle near a random edge.
    /// The circle is placed randomly near one of the four edges.
    pub fn spawn_food_circle(&mut self, radius: f32) {
        use rand::RngExt;
        let mut rng = rand::rng();

        // Pick a random edge: 0 = top, 1 = right, 2 = bottom, 3 = left
        let edge: u32 = (rng.random::<f32>() * 4.0).floor() as u32;

        let center = match edge {
            0 => {
                // Top edge
                let x: f32 = rng.random::<f32>();
                [x, radius + 0.01] // Slightly offset from edge
            }
            1 => {
                // Right edge
                let y: f32 = rng.random::<f32>();
                [1.0 - radius - 0.01, y]
            }
            2 => {
                // Bottom edge
                let x: f32 = rng.random::<f32>();
                [x, 1.0 - radius - 0.01]
            }
            3 => {
                // Left edge
                let y: f32 = rng.random::<f32>();
                [radius + 0.01, y]
            }
            _ => [0.5, 0.5],
        };

        self.food_grid.fill_circle(center, radius);
    }
}

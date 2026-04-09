use crate::ant::{Ant, AntType};

/// Colony settings shared between CPU logic and GPU shaders.
/// Layout must match the `Colony` struct in compute.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Colony {
    pub center: [f32; 2],
    pub half_size: f32,
    pub _pad: f32,
}

impl Colony {
    pub fn new(center: [f32; 2], half_size: f32) -> Self {
        Self { center, half_size, _pad: 0.0 }
    }
}

impl Default for Colony {
    fn default() -> Self {
        Self::new([0.2, 0.2], 0.1)
    }
}

#[derive(Debug)]
pub struct Spawner {
    pub colony: Colony,
    pub ant_count: usize,
    pub ants_per_second: f32,
    active_count: u32,
    last_tick: std::time::Instant,
    spawn_accumulator: f32,
}

impl Spawner {
    pub fn new(colony: Colony, ant_count: usize, ants_per_second: f32) -> Self {
        Self {
            colony,
            ant_count,
            ants_per_second,
            active_count: 1,
            last_tick: std::time::Instant::now(),
            spawn_accumulator: 0.0,
        }
    }

    pub fn initial_ants(&self) -> Vec<Ant> {
        use rand::RngExt;
        use std::f32::consts::TAU;

        const BASE_SPEED: f32 = 0.0002;
        const SPEED_VARIATION: f32 = 0.0001;

        let mut rng = rand::rng();
        let [cx, cy] = self.colony.center;
        let hs = self.colony.half_size;

        (0..self.ant_count)
            .map(|i| {
                let angle = rng.random::<f32>() * TAU;
                let speed = BASE_SPEED + rng.random_range(-SPEED_VARIATION..SPEED_VARIATION);
                let ant_type = if i % 10 == 0 { AntType::Scout } else { AntType::Forager };
                let x = cx + rng.random_range(-hs..hs);
                let y = cy + rng.random_range(-hs..hs);
                Ant::new([x, y], [angle.cos() * speed, angle.sin() * speed], ant_type)
            })
            .collect()
    }

    pub fn active_count(&self) -> u32 {
        self.active_count
    }

    /// Advance the spawn timer and return the updated active ant count.
    pub fn tick(&mut self) -> u32 {
        let now = std::time::Instant::now();
        let delta = now.duration_since(self.last_tick).as_secs_f32();
        self.last_tick = now;
        self.spawn_accumulator += delta * self.ants_per_second;
        let new_ants = self.spawn_accumulator.floor() as u32;
        self.spawn_accumulator -= new_ants as f32;
        self.active_count = (self.active_count + new_ants).min(self.ant_count as u32);
        self.active_count
    }
}

impl Default for Spawner {
    fn default() -> Self {
        Self::new(Colony::default(), 1000, 10.0)
    }
}

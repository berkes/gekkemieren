use crate::ant::{Ant, AntType};

const BASE_SPEED: f32 = 0.0015;
pub const N_ANTS: usize = 15000;

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
    pub scout_ratio: f32,
}

impl Spawner {
    pub fn new(colony: Colony, ant_count: usize, scout_ratio: f32) -> Self {
        Self { colony, ant_count, scout_ratio }
    }

    pub fn initial_ants(&self) -> Vec<Ant> {
        use rand::RngExt;
        use std::f32::consts::TAU;

        let mut rng = rand::rng();
        let [cx, cy] = self.colony.center;
        let hs = self.colony.half_size;

        (0..self.ant_count)
            .map(|_| {
                let angle = rng.random::<f32>() * TAU;
                let speed = BASE_SPEED;
                let ant_type = if rng.random::<f32>() < self.scout_ratio {
                    AntType::Scout
                } else {
                    AntType::Forager
                };
                let x = cx + rng.random_range(-hs..hs);
                let y = cy + rng.random_range(-hs..hs);
                Ant::new([x, y], [angle.cos() * speed, angle.sin() * speed], ant_type)
            })
            .collect()
    }
}

impl Default for Spawner {
    fn default() -> Self {
        Self::new(Colony::default(), N_ANTS, 0.1)
    }
}

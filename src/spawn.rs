use crate::ant::{Ant, AntType};

pub trait AntSpawner {
    fn ants(&self) -> &[Ant];
    fn colony(&self) -> &Colony;
    fn ant_count(&self) -> usize {
        self.ants().len()
    }
}

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
        Self {
            center,
            half_size,
            _pad: 0.0,
        }
    }
}

impl Default for Colony {
    fn default() -> Self {
        Self::new([0.2, 0.2], 0.1)
    }
}

/// Spawns ants at random positions within the colony bounds.
/// Ants are generated once at construction and held for GPU upload.
#[derive(Debug)]
pub struct RandomSpawner {
    pub colony: Colony,
    ants: Vec<Ant>,
}

impl RandomSpawner {
    pub fn new(colony: Colony, ant_count: usize, scout_ratio: f32, speed: f32) -> Self {
        use rand::RngExt;
        use std::f32::consts::TAU;

        let mut rng = rand::rng();
        let [cx, cy] = colony.center;
        let hs = colony.half_size;

        let ants = (0..ant_count)
            .map(|_| {
                let angle = rng.random::<f32>() * TAU;
                let ant_type = if rng.random::<f32>() < scout_ratio {
                    AntType::Scout
                } else {
                    AntType::Forager
                };
                let x = cx + rng.random_range(-hs..hs);
                let y = cy + rng.random_range(-hs..hs);
                Ant::new([x, y], [angle.cos() * speed, angle.sin() * speed], ant_type)
            })
            .collect();

        Self { colony, ants }
    }
}

impl AntSpawner for RandomSpawner {
    fn ants(&self) -> &[Ant] {
        &self.ants
    }

    fn colony(&self) -> &Colony {
        &self.colony
    }
}

/// Spawns a fixed, caller-provided list of ants. Used for deterministic tests.
#[derive(Debug)]
pub struct FixedSpawner {
    pub ants: Vec<Ant>,
    pub colony: Colony,
}

impl FixedSpawner {
    pub fn new(ants: Vec<Ant>, colony: Colony) -> Self {
        Self { ants, colony }
    }
}

impl AntSpawner for FixedSpawner {
    fn ants(&self) -> &[Ant] {
        &self.ants
    }

    fn colony(&self) -> &Colony {
        &self.colony
    }
}

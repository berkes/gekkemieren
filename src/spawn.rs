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

#[cfg(test)]
mod tests {
    use super::*;

    fn spawner() -> RandomSpawner {
        RandomSpawner::new(Colony::new([0.5, 0.5], 0.1), 1000, 0.2, 0.002)
    }

    #[test]
    fn ant_count_matches_requested() {
        let s = spawner();
        assert_eq!(s.ant_count(), 1000);
        assert_eq!(s.ants().len(), 1000);
    }

    #[test]
    fn all_ants_within_colony_bounds() {
        let s = spawner();
        let [cx, cy] = s.colony.center;
        let hs = s.colony.half_size;
        for ant in s.ants() {
            let [x, y] = ant.position;
            assert!(
                (x - cx).abs() < hs && (y - cy).abs() < hs,
                "ant at [{x}, {y}] is outside colony bounds"
            );
        }
    }

    #[test]
    fn scout_ratio_is_approximately_correct() {
        let s = spawner();
        let scout_count = s.ants().iter().filter(|a| a.ant_type == 1).count();
        let actual_ratio = scout_count as f32 / s.ant_count() as f32;
        // Allow ±5% deviation from the requested 20% ratio.
        assert!(
            (actual_ratio - 0.2).abs() < 0.05,
            "scout ratio {actual_ratio:.3} is too far from 0.2"
        );
    }

    #[test]
    fn all_ants_have_correct_speed() {
        let s = spawner();
        for ant in s.ants() {
            let [dx, dy] = ant.direction;
            let speed = (dx * dx + dy * dy).sqrt();
            assert!(
                (speed - 0.002).abs() < 1e-6,
                "ant speed {speed} differs from requested 0.002"
            );
        }
    }
}

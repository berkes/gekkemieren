use crate::ant::{Ant, AntType};

pub trait AntSpawner {
    fn ants(&self) -> &[Ant];
    fn ants_mut(&mut self) -> &mut [Ant];
    fn colony(&self) -> &Colony;
    fn ant_count(&self) -> usize {
        self.ants().len()
    }

    /// Adjusts the scout ratio by converting ants from one type to another.
    /// Keeps position and direction of converted ants intact.
    fn adjust_scout_ratio(&mut self, new_ratio: f32) {
        use rand::prelude::SliceRandom;
        use rand::rng;

        let ants = self.ants_mut();
        let total = ants.len();
        if total == 0 {
            return;
        }

        let target_scout_count = (new_ratio.clamp(0.0, 1.0) * total as f32).round() as usize;
        let current_scout_count = ants.iter().filter(|a| a.ant_type == 1).count();

        if target_scout_count == current_scout_count {
            return;
        }

        // Collect indices of ants that can be converted
        let mut forager_indices: Vec<usize> = ants
            .iter()
            .enumerate()
            .filter(|(_, a)| a.ant_type == 0)
            .map(|(i, _)| i)
            .collect();
        let mut scout_indices: Vec<usize> = ants
            .iter()
            .enumerate()
            .filter(|(_, a)| a.ant_type == 1)
            .map(|(i, _)| i)
            .collect();

        // Shuffle to pick random ants
        let mut rng = rng();
        forager_indices.shuffle(&mut rng);
        scout_indices.shuffle(&mut rng);

        if target_scout_count > current_scout_count {
            // Need more Scouts: convert Foragers to Scouts
            let to_convert = target_scout_count - current_scout_count;
            for &idx in forager_indices.iter().take(to_convert) {
                ants[idx].ant_type = 1; // Scout
            }
        } else {
            // Need fewer Scouts: convert Scouts to Foragers
            let to_convert = current_scout_count - target_scout_count;
            for &idx in scout_indices.iter().take(to_convert) {
                ants[idx].ant_type = 0; // Forager
            }
        }
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
        Self::new([0.5, 0.5], 0.01)
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

    fn ants_mut(&mut self) -> &mut [Ant] {
        &mut self.ants
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

    fn ants_mut(&mut self) -> &mut [Ant] {
        &mut self.ants
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
    fn adjust_scout_ratio_converts_ants_preserving_position_and_direction() {
        // Create 10 ants: 8 Foragers + 2 Scouts with known positions and directions
        let careful_ants = (0..8)
            .map(|i| Ant::new([i as f32 * 0.01, 0.0], [1.0, 0.0], AntType::Forager))
            .chain(
                (0..2).map(|i| Ant::new([0.0, (i + 1) as f32 * 0.01], [0.0, 1.0], AntType::Scout)),
            )
            .collect::<Vec<_>>();

        let colony = Colony::new([0.5, 0.5], 0.1);
        let mut spawner = FixedSpawner::new(careful_ants, colony);

        // Store original positions and directions of all ants
        let original_ants: Vec<_> = spawner
            .ants()
            .iter()
            .map(|a| (a.position, a.direction))
            .collect();

        // Adjust ratio from 0.2 to 0.5 (need 5 Scouts, have 2, so convert 3 Foragers)
        spawner.adjust_scout_ratio(0.5);

        // Verify new ratio is approximately 0.5
        let scout_count = spawner.ants().iter().filter(|a| a.ant_type == 1).count();
        let forager_count = spawner.ants().len() - scout_count;
        assert_eq!(scout_count, 5, "Expected 5 Scouts after adjustment");
        assert_eq!(forager_count, 5, "Expected 5 Foragers after adjustment");

        // Verify positions and directions are preserved
        for (i, ant) in spawner.ants().iter().enumerate() {
            let (expected_pos, expected_dir) = original_ants[i];
            assert_eq!(
                ant.position, expected_pos,
                "Position changed for ant {} during conversion",
                i
            );
            assert_eq!(
                ant.direction, expected_dir,
                "Direction changed for ant {} during conversion",
                i
            );
        }
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

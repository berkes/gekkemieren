pub enum AntType {
    Forager,
    Scout,
}

impl AntType {
    fn as_u32(&self) -> u32 {
        match self {
            AntType::Forager => 0,
            AntType::Scout => 1,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Ant {
    pub position: [f32; 2],
    pub direction: [f32; 2],
    pub ant_type: u32,
    /// 0 = inside colony (collision suppressed), 1 = has emerged
    pub emerged: u32,
}

impl Ant {
    pub fn new(position: [f32; 2], direction: [f32; 2], ant_type: AntType) -> Self {
        Self {
            position,
            direction,
            ant_type: ant_type.as_u32(),
            emerged: 0,
        }
    }
}

pub fn initial_ants(count: usize) -> Vec<Ant> {
    use rand::RngExt;
    use std::f32::consts::TAU;

    const COLONY_HALF_SIZE: f32 = 0.02;
    const BASE_SPEED: f32 = 0.0002;
    const SPEED_VARIATION: f32 = 0.0001;

    let mut rng = rand::rng();
    (0..count)
        .map(|i| {
            let angle = rng.random::<f32>() * TAU;
            let speed = BASE_SPEED + rng.random_range(-SPEED_VARIATION..SPEED_VARIATION);
            let ant_type = if i % 10 == 0 {
                AntType::Scout
            } else {
                AntType::Forager
            };
            let x = 0.5 + rng.random_range(-COLONY_HALF_SIZE..COLONY_HALF_SIZE);
            let y = 0.5 + rng.random_range(-COLONY_HALF_SIZE..COLONY_HALF_SIZE);
            Ant::new([x, y], [angle.cos() * speed, angle.sin() * speed], ant_type)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_ants_returns_correct_count() {
        let ants = initial_ants(100);
        assert_eq!(ants.len(), 100);
    }

    #[test]
    fn ant_positions_are_in_unit_range() {
        let ants = initial_ants(100);
        for ant in &ants {
            assert!(ant.position[0] >= 0.0 && ant.position[0] <= 1.0);
            assert!(ant.position[1] >= 0.0 && ant.position[1] <= 1.0);
        }
    }

    #[test]
    fn ant_type_encodes_correctly() {
        let forager = Ant::new([0.0, 0.0], [0.0, 0.0], AntType::Forager);
        let scout = Ant::new([0.0, 0.0], [0.0, 0.0], AntType::Scout);
        assert_eq!(forager.ant_type, 0);
        assert_eq!(scout.ant_type, 1);
    }
}

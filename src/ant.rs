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
    pub _pad: u32,
}

impl Ant {
    pub fn new(position: [f32; 2], direction: [f32; 2], ant_type: AntType) -> Self {
        Self {
            position,
            direction,
            ant_type: ant_type.as_u32(),
            _pad: 0,
        }
    }
}

pub fn initial_ants(count: usize) -> Vec<Ant> {
    use std::f32::consts::FRAC_PI_2;
    (0..count)
        .map(|i| {
            let angle = (i as f32 / count as f32) * FRAC_PI_2;
            let ant_type = if i % 10 == 0 {
                AntType::Scout
            } else {
                AntType::Forager
            };
            Ant::new(
                [0.0, 0.0],
                [angle.cos() * 0.0002, angle.sin() * 0.0002],
                ant_type,
            )
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

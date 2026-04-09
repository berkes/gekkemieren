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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ant_type_encodes_correctly() {
        let forager = Ant::new([0.0, 0.0], [0.0, 0.0], AntType::Forager);
        let scout = Ant::new([0.0, 0.0], [0.0, 0.0], AntType::Scout);
        assert_eq!(forager.ant_type, 0);
        assert_eq!(scout.ant_type, 1);
    }
}

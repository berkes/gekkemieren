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
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Ant {
    pub position: [f32; 2],
    pub direction: [f32; 2],
    pub ant_type: u32,
    pub carries_food: u32,
}

impl Ant {
    pub fn new(position: [f32; 2], direction: [f32; 2], ant_type: AntType) -> Self {
        Self {
            position,
            direction,
            ant_type: ant_type.as_u32(),
            carries_food: 0,
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

    #[test]
    fn ant_struct_size_and_offsets() {
        // Verify Ant struct is 24 bytes (8+8+4+4) with no padding
        assert_eq!(std::mem::size_of::<Ant>(), 24);

        // Verify field offsets match WGSL std430 layout
        assert_eq!(std::mem::offset_of!(Ant, position), 0);
        assert_eq!(std::mem::offset_of!(Ant, direction), 8);
        assert_eq!(std::mem::offset_of!(Ant, ant_type), 16);
        assert_eq!(std::mem::offset_of!(Ant, carries_food), 20);
    }

    #[test]
    fn new_ant_initializes_carries_food_to_zero() {
        let ant = Ant::new([0.5, 0.5], [1.0, 0.0], AntType::Forager);
        assert_eq!(ant.carries_food, 0);
    }

    #[test]
    fn carries_food_offset_is_20() {
        // Create an ant and verify the carries_food field is at offset 20
        let ant = Ant {
            position: [1.0, 2.0],
            direction: [3.0, 4.0],
            ant_type: 5,
            carries_food: 0xDEADBEEF,
        };
        let bytes: &[u8] = bytemuck::bytes_of(&ant);
        // offset 0-8: position [1.0, 2.0] as f32
        // offset 8-16: direction [3.0, 4.0] as f32
        // offset 16-20: ant_type = 5 as u32 (little-endian: 05 00 00 00)
        // offset 20-24: carries_food = 0xDEADBEEF as u32 (little-endian: EF BE AD DE)
        assert_eq!(bytes[20..24], [0xEF, 0xBE, 0xAD, 0xDE]);
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridInfo {
    pub width: u32,
    pub height: u32,
    pub _pad: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimConfig {
    pub decay_amount: u32,
    pub max_strength: u32,
    pub deposit_amount: u32,
    pub dot_radius: f32,
    pub collision_radius: f32,
    pub collision_angle_min: f32,
    pub collision_angle_max: f32,
    pub forager_randomness: f32,
    pub scout_randomness: f32,
    pub _pad: [u32; 3],
}

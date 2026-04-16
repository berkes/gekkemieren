#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GridInfo {
    pub width: u32,
    pub height: u32,
    pub _pad: [u32; 2],
}

// use image::GenericImageView;
use wgpu::{self, util::DeviceExt};

const VERTICES: &[Vertex] = &[
    Vertex { // 0
        position: [0.0, 0.3, 0.0],
        color: [1.0, 1.0, 1.0],
    },
    Vertex { // 1
        position: [-0.1, 0.4, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex { // 2
        position: [-0.3, 0.4, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex { // 3 right
        position: [-0.4, 0.2, 0.0],
        color: [1.0, 1.0, 1.0],
    },
    Vertex { // 4, center
        position: [0.0, -0.4, 0.0],
        color: [0.0, 0.0, 1.0],
    },
    Vertex { // 5 left
        position: [0.4, 0.2, 0.0],
        color: [1.0, 1.0, 1.0],
    },
    Vertex { // 6
        position: [0.3, 0.4, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    Vertex { // 7
        position: [0.1, 0.4, 0.0],
        color: [1.0, 0.0, 0.0],
    },
];

pub const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
    5, 6, 4,
    6, 7, 4,
    7, 0, 4,
];

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[derive(Debug)]
pub struct ShaderManager {
    device: wgpu::Device,
}

impl ShaderManager {
    pub fn new(device: wgpu::Device) -> Self {
        Self { device }
    }

    pub fn create_shader_module(&self) -> wgpu::ShaderModule {
        self.device
            .create_shader_module(wgpu::include_wgsl!("shader.wgsl"))
    }

    pub fn create_render_pipeline(
        &self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::RenderPipeline {
        let shader = self.create_shader_module();
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        })
    }

    pub(crate) fn create_buffers(&self) -> (wgpu::Buffer, wgpu::Buffer) {
        (
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                }),
        )
    }

    // pub(crate) fn create_texture(&self, diffuse_bytes: &[u8]) -> wgpu::Texture  {
    //     let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
    //     let diffuse_rgba = diffuse_image.to_rgba8();
    //     let dimensions = diffuse_image.dimensions();
    //     let texture_size = wgpu::Extent3d {
    //         width: dimensions.0,
    //         height: dimensions.1,
    //         depth_or_array_layers: 1,
    //     };

    //     self.device
    //         .create_texture(&wgpu::TextureDescriptor {
    //             label: Some("Diffuse Texture"),
    //             size: texture_size,
    //             mip_level_count: 1,
    //             sample_count: 1,
    //             dimension: wgpu::TextureDimension::D2,
    //             format: wgpu::TextureFormat::Rgba8Unorm,
    //             usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    //             view_formats: &[],
    //         })
    // }
}

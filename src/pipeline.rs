use wgpu::util::DeviceExt;

use crate::spawn::Spawner;

#[derive(Debug)]
pub struct Pipeline {
    collision_pipeline: wgpu::ComputePipeline,
    collision_bind_group: wgpu::BindGroup,
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group: wgpu::BindGroup,
    spawner: Spawner,
}

impl Pipeline {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> anyhow::Result<Self> {
        let spawner = Spawner::default();
        let ants = spawner.initial_ants();

        let ant_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ant_buffer"),
            contents: bytemuck::cast_slice(&ants),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let colony_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("colony_buffer"),
            contents: bytemuck::bytes_of(&spawner.colony),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let compute_shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/compute.wgsl"));
        let collision_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("collision_pipeline"),
                layout: None,
                module: &compute_shader,
                entry_point: Some("collision_main"),
                compilation_options: Default::default(),
                cache: Default::default(),
            });
        let collision_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("collision_bind_group"),
            layout: &collision_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ant_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: colony_buffer.as_entire_binding(),
                },
            ],
        });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("compute_pipeline"),
            layout: None,
            module: &compute_shader,
            entry_point: Some("movement_main"),
            compilation_options: Default::default(),
            cache: Default::default(),
        });
        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("compute_bind_group"),
            layout: &compute_pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: ant_buffer.as_entire_binding(),
            }],
        });

        let render_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/render.wgsl"));
        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("render_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("render_bind_group"),
            layout: &render_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: ant_buffer.as_entire_binding(),
            }],
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts: &[Some(&render_bind_group_layout)],
                immediate_size: 0,
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &render_shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &render_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Ok(Self {
            collision_pipeline,
            collision_bind_group,
            compute_pipeline,
            compute_bind_group,
            render_pipeline,
            render_bind_group,
            spawner,
        })
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let ant_count = self.spawner.ant_count as u32;
        let mut encoder = device.create_command_encoder(&Default::default());
        let dispatches = ant_count.div_ceil(64);
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.collision_pipeline);
            pass.set_bind_group(0, &self.collision_bind_group, &[]);
            pass.dispatch_workgroups(dispatches, 1, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.compute_pipeline);
            pass.set_bind_group(0, &self.compute_bind_group, &[]);
            pass.dispatch_workgroups(dispatches, 1, 1);
        }
        queue.submit([encoder.finish()]);
    }

    pub fn draw<'pass>(&'pass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.render_bind_group, &[]);
        render_pass.draw(0..6, 0..self.spawner.ant_count as u32);
    }
}

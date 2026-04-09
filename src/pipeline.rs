use wgpu::util::DeviceExt;

use crate::pheromone::{GridInfo, SimConfig};
use crate::spawn::Spawner;

#[derive(Debug)]
pub struct Pipeline {
    collision_pipeline: wgpu::ComputePipeline,
    collision_bind_group: wgpu::BindGroup,
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group: wgpu::BindGroup,
    pheromone_decay_pipeline: wgpu::ComputePipeline,
    pheromone_decay_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group: wgpu::BindGroup,
    pheromone_render_pipeline: wgpu::RenderPipeline,
    pheromone_render_bind_group: wgpu::BindGroup,
    ant_buffer: wgpu::Buffer,
    grid_info_buffer: wgpu::Buffer,
    pheromone_buffer: wgpu::Buffer,
    config_buffer: wgpu::Buffer,
    spawner: Spawner,
    grid_width: u32,
    grid_height: u32,
}

fn create_pheromone_buffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Buffer {
    let data = vec![0u32; (width * height) as usize];
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("pheromone_buffer"),
        contents: bytemuck::cast_slice(&data),
        usage: wgpu::BufferUsages::STORAGE,
    })
}

fn create_compute_bind_group(
    device: &wgpu::Device,
    pipeline: &wgpu::ComputePipeline,
    ant_buffer: &wgpu::Buffer,
    pheromone_buffer: &wgpu::Buffer,
    grid_info_buffer: &wgpu::Buffer,
    config_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("compute_bind_group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: ant_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: pheromone_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: grid_info_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 4, resource: config_buffer.as_entire_binding() },
        ],
    })
}

fn create_pheromone_decay_bind_group(
    device: &wgpu::Device,
    pipeline: &wgpu::ComputePipeline,
    pheromone_buffer: &wgpu::Buffer,
    config_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("pheromone_decay_bind_group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: pheromone_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: config_buffer.as_entire_binding() },
        ],
    })
}

fn create_pheromone_render_bind_group(
    device: &wgpu::Device,
    pipeline: &wgpu::RenderPipeline,
    pheromone_buffer: &wgpu::Buffer,
    grid_info_buffer: &wgpu::Buffer,
    config_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("pheromone_render_bind_group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: pheromone_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: grid_info_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: config_buffer.as_entire_binding() },
        ],
    })
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        sim_config: SimConfig,
    ) -> anyhow::Result<Self> {
        let spawner = Spawner::default();
        let ants = spawner.initial_ants();
        let grid_width = config.width;
        let grid_height = config.height;

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
        let grid_info = GridInfo { width: grid_width, height: grid_height, _pad: [0; 2] };
        let grid_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("grid_info_buffer"),
            contents: bytemuck::bytes_of(&grid_info),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let pheromone_buffer = create_pheromone_buffer(device, grid_width, grid_height);
        let config_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("config_buffer"),
            contents: bytemuck::bytes_of(&sim_config),
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
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: config_buffer.as_entire_binding(),
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
        let compute_bind_group = create_compute_bind_group(
            device,
            &compute_pipeline,
            &ant_buffer,
            &pheromone_buffer,
            &grid_info_buffer,
            &config_buffer,
        );

        let pheromone_decay_shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/pheromone_decay.wgsl"));
        let pheromone_decay_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("pheromone_decay_pipeline"),
                layout: None,
                module: &pheromone_decay_shader,
                entry_point: Some("pheromone_decay_main"),
                compilation_options: Default::default(),
                cache: Default::default(),
            });
        let pheromone_decay_bind_group = create_pheromone_decay_bind_group(
            device,
            &pheromone_decay_pipeline,
            &pheromone_buffer,
            &config_buffer,
        );

        let render_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/render.wgsl"));
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: None,
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

        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("render_bind_group"),
            layout: &render_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: ant_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: config_buffer.as_entire_binding() },
            ],
        });

        let pheromone_render_shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/pheromone_render.wgsl"));
        let pheromone_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("pheromone_render_pipeline"),
                layout: None,
                vertex: wgpu::VertexState {
                    module: &pheromone_render_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &pheromone_render_shader,
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
        let pheromone_render_bind_group = create_pheromone_render_bind_group(
            device,
            &pheromone_render_pipeline,
            &pheromone_buffer,
            &grid_info_buffer,
            &config_buffer,
        );

        Ok(Self {
            collision_pipeline,
            collision_bind_group,
            compute_pipeline,
            compute_bind_group,
            pheromone_decay_pipeline,
            pheromone_decay_bind_group,
            render_pipeline,
            render_bind_group,
            pheromone_render_pipeline,
            pheromone_render_bind_group,
            ant_buffer,
            grid_info_buffer,
            pheromone_buffer,
            config_buffer,
            spawner,
            grid_width,
            grid_height,
        })
    }

    pub fn resize(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) {
        self.grid_width = width;
        self.grid_height = height;

        let grid_info = GridInfo { width, height, _pad: [0; 2] };
        queue.write_buffer(&self.grid_info_buffer, 0, bytemuck::bytes_of(&grid_info));

        self.pheromone_buffer = create_pheromone_buffer(device, width, height);
        self.compute_bind_group = create_compute_bind_group(
            device,
            &self.compute_pipeline,
            &self.ant_buffer,
            &self.pheromone_buffer,
            &self.grid_info_buffer,
            &self.config_buffer,
        );
        self.pheromone_decay_bind_group = create_pheromone_decay_bind_group(
            device,
            &self.pheromone_decay_pipeline,
            &self.pheromone_buffer,
            &self.config_buffer,
        );
        self.pheromone_render_bind_group = create_pheromone_render_bind_group(
            device,
            &self.pheromone_render_pipeline,
            &self.pheromone_buffer,
            &self.grid_info_buffer,
            &self.config_buffer,
        );
    }

    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let ant_count = self.spawner.ant_count as u32;
        let pheromone_count = self.grid_width * self.grid_height;
        let mut encoder = device.create_command_encoder(&Default::default());

        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.pheromone_decay_pipeline);
            pass.set_bind_group(0, &self.pheromone_decay_bind_group, &[]);
            pass.dispatch_workgroups(pheromone_count.div_ceil(64), 1, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.collision_pipeline);
            pass.set_bind_group(0, &self.collision_bind_group, &[]);
            pass.dispatch_workgroups(ant_count.div_ceil(64), 1, 1);
        }
        {
            let mut pass = encoder.begin_compute_pass(&Default::default());
            pass.set_pipeline(&self.compute_pipeline);
            pass.set_bind_group(0, &self.compute_bind_group, &[]);
            pass.dispatch_workgroups(ant_count.div_ceil(64), 1, 1);
        }

        queue.submit([encoder.finish()]);
    }

    pub fn draw<'pass>(&'pass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        render_pass.set_pipeline(&self.pheromone_render_pipeline);
        render_pass.set_bind_group(0, &self.pheromone_render_bind_group, &[]);
        render_pass.draw(0..6, 0..1);

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.render_bind_group, &[]);
        render_pass.draw(0..6, 0..self.spawner.ant_count as u32);
    }
}

use wgpu::util::DeviceExt;

use crate::ant::Ant;
use crate::color_scheme::ColorScheme;
use crate::config::GpuConfig;
use crate::pheromone::GridInfo;
use crate::spawn::Colony;

// ── helpers shared by both pipeline types ────────────────────────────────────

fn create_pheromone_buffer(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Buffer {
    let data = vec![0u32; (width * height) as usize];
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("pheromone_buffer"),
        contents: bytemuck::cast_slice(&data),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
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
            wgpu::BindGroupEntry {
                binding: 0,
                resource: ant_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: pheromone_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: grid_info_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: config_buffer.as_entire_binding(),
            },
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
            wgpu::BindGroupEntry {
                binding: 0,
                resource: pheromone_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: config_buffer.as_entire_binding(),
            },
        ],
    })
}

fn create_pheromone_render_bind_group(
    device: &wgpu::Device,
    pipeline: &wgpu::RenderPipeline,
    pheromone_buffer: &wgpu::Buffer,
    grid_info_buffer: &wgpu::Buffer,
    config_buffer: &wgpu::Buffer,
    color_scheme_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("pheromone_render_bind_group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: pheromone_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: grid_info_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: config_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: color_scheme_buffer.as_entire_binding(),
            },
        ],
    })
}

// ── SimulationPipeline ────────────────────────────────────────────────────────

/// Compute pipelines and simulation state buffers.
/// No dependency on windowing or surface format — safe for headless use.
#[derive(Debug)]
pub struct SimulationPipeline {
    collision_pipeline: wgpu::ComputePipeline,
    collision_bind_group: wgpu::BindGroup,
    compute_pipeline: wgpu::ComputePipeline,
    compute_bind_group: wgpu::BindGroup,
    pheromone_decay_pipeline: wgpu::ComputePipeline,
    pheromone_decay_bind_group: wgpu::BindGroup,

    pub ant_buffer: wgpu::Buffer,
    pub pheromone_buffer: wgpu::Buffer,
    pub grid_info_buffer: wgpu::Buffer,
    pub config_buffer: wgpu::Buffer,

    pub ant_count: usize,
    pub grid_width: u32,
    pub grid_height: u32,
}

impl SimulationPipeline {
    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        config: GpuConfig,
        colony: &Colony,
        ants: &[Ant],
    ) -> Self {
        let ant_count = ants.len();

        let ant_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ant_buffer"),
            contents: bytemuck::cast_slice(ants),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        });
        let colony_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("colony_buffer"),
            contents: bytemuck::bytes_of(colony),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let grid_info = GridInfo {
            width,
            height,
            _pad: [0; 2],
        };
        let grid_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("grid_info_buffer"),
            contents: bytemuck::bytes_of(&grid_info),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let pheromone_buffer = create_pheromone_buffer(device, width, height);
        let config_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("config_buffer"),
            contents: bytemuck::bytes_of(&config),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let compute_shader =
            device.create_shader_module(wgpu::include_wgsl!("shaders/compute.wgsl"));

        let collision_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
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

        Self {
            collision_pipeline,
            collision_bind_group,
            compute_pipeline,
            compute_bind_group,
            pheromone_decay_pipeline,
            pheromone_decay_bind_group,
            ant_buffer,
            pheromone_buffer,
            grid_info_buffer,
            config_buffer,
            ant_count,
            grid_width: width,
            grid_height: height,
        }
    }

    /// Dispatches one simulation tick: pheromone decay → collision → movement.
    pub fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let ant_count = self.ant_count as u32;
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

    /// Resizes the pheromone grid and rebuilds the affected bind groups.
    pub fn resize(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, width: u32, height: u32) {
        self.grid_width = width;
        self.grid_height = height;

        let grid_info = GridInfo {
            width,
            height,
            _pad: [0; 2],
        };
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
    }

    /// Copies the ant buffer back to CPU. Used in tests to inspect simulation state.
    pub fn read_ant_state(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<Ant> {
        let size = (self.ant_count * std::mem::size_of::<Ant>()) as u64;
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ant_staging"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(&self.ant_buffer, 0, &staging, 0, size);
        queue.submit([encoder.finish()]);

        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .unwrap();

        let data = slice.get_mapped_range();
        let result: Vec<Ant> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging.unmap();
        result
    }

    /// Copies the pheromone buffer back to CPU. Used in tests to inspect pheromone state.
    pub fn read_pheromone_state(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> Vec<u32> {
        let size = (self.grid_width * self.grid_height * std::mem::size_of::<u32>() as u32) as u64;
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pheromone_staging"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(&self.pheromone_buffer, 0, &staging, 0, size);
        queue.submit([encoder.finish()]);

        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .unwrap();

        let data = slice.get_mapped_range();
        let result: Vec<u32> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging.unmap();
        result
    }
}

// ── RenderPipeline ────────────────────────────────────────────────────────────

/// Render pipelines for displaying ants and pheromones on screen.
/// Reads from buffers owned by `SimulationPipeline`.
#[derive(Debug)]
pub struct RenderPipeline {
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group: wgpu::BindGroup,
    pheromone_render_pipeline: wgpu::RenderPipeline,
    pheromone_render_bind_group: wgpu::BindGroup,
    color_scheme_buffer: wgpu::Buffer,
    pub background_color: wgpu::Color,
}

impl RenderPipeline {
    pub fn new(
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
        simulation: &SimulationPipeline,
        color_scheme: ColorScheme,
    ) -> anyhow::Result<Self> {
        let color_scheme_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("color_scheme_buffer"),
            contents: bytemuck::bytes_of(&color_scheme),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let background_color = wgpu::Color {
            r: color_scheme.background[0] as f64,
            g: color_scheme.background[1] as f64,
            b: color_scheme.background[2] as f64,
            a: color_scheme.background[3] as f64,
        };

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
                    format: texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::PointList,
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
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: simulation.ant_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: color_scheme_buffer.as_entire_binding(),
                },
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
                        format: texture_format,
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
            &simulation.pheromone_buffer,
            &simulation.grid_info_buffer,
            &simulation.config_buffer,
            &color_scheme_buffer,
        );

        Ok(Self {
            render_pipeline,
            render_bind_group,
            pheromone_render_pipeline,
            pheromone_render_bind_group,
            color_scheme_buffer,
            background_color,
        })
    }

    pub fn set_color_scheme(&mut self, queue: &wgpu::Queue, scheme: ColorScheme) {
        queue.write_buffer(&self.color_scheme_buffer, 0, bytemuck::bytes_of(&scheme));
        self.background_color = wgpu::Color {
            r: scheme.background[0] as f64,
            g: scheme.background[1] as f64,
            b: scheme.background[2] as f64,
            a: scheme.background[3] as f64,
        };
    }

    /// Rebuilds the pheromone render bind group after a simulation resize.
    /// Must be called whenever `SimulationPipeline::resize` has been called.
    pub fn on_resize(&mut self, device: &wgpu::Device, simulation: &SimulationPipeline) {
        self.pheromone_render_bind_group = create_pheromone_render_bind_group(
            device,
            &self.pheromone_render_pipeline,
            &simulation.pheromone_buffer,
            &simulation.grid_info_buffer,
            &simulation.config_buffer,
            &self.color_scheme_buffer,
        );
    }

    pub fn draw<'pass>(&'pass self, render_pass: &mut wgpu::RenderPass<'pass>, ant_count: u32) {
        render_pass.set_pipeline(&self.pheromone_render_pipeline);
        render_pass.set_bind_group(0, &self.pheromone_render_bind_group, &[]);
        render_pass.draw(0..6, 0..1);

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.render_bind_group, &[]);
        render_pass.draw(0..1, 0..ant_count);
    }
}

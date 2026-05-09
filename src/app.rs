use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{
    color_scheme::ColorScheme,
    config::{Config, GpuConfig},
    food::FoodSpawner,
    pipeline::{RenderPipeline, SimulationPipeline},
    screenshot::{save_screenshot, save_state},
    spawn::{AntSpawner, Colony, RandomSpawner},
    wgpu_setup::WgpuSetup,
};

#[derive(Debug)]
pub struct State {
    window: Arc<Window>,
    wgpu_setup: WgpuSetup,
    simulation: SimulationPipeline,
    pipeline: RenderPipeline,
    config: Config,
    is_surface_configured: bool,
    frame_count: u32,
    log_timer: std::time::Instant,
    spawner: RandomSpawner,
    food_spawner: FoodSpawner,
}

impl State {
    async fn new(window: Arc<Window>, config: Config) -> Result<Self> {
        let wgpu_setup = WgpuSetup::new(window.clone()).await?;

        let gpu_config = GpuConfig::from(&config);

        let spawner = RandomSpawner::new(
            Colony::default(),
            config.n_ants,
            config.scout_ratio,
            config.base_speed,
        );

        // Initialize food spawner and spawn food near a random edge
        let mut food_spawner = FoodSpawner::new(wgpu_setup.config.width, wgpu_setup.config.height);
        food_spawner.spawn_food_circle(config.food_source_radius); // Spawn a circle with radius 0.01 (1% of screen)

        let simulation = SimulationPipeline::new(
            &wgpu_setup.device,
            wgpu_setup.config.width,
            wgpu_setup.config.height,
            gpu_config,
            spawner.colony(),
            spawner.ants(),
        );

        // Upload initial food data to GPU buffer
        wgpu_setup.queue.write_buffer(
            &simulation.food_buffer,
            0,
            bytemuck::cast_slice(&food_spawner.food_grid.data),
        );

        let color_scheme = ColorScheme::from_palette(config.palette);
        let pipeline = RenderPipeline::new(
            &wgpu_setup.device,
            wgpu_setup.config.format,
            &simulation,
            color_scheme,
        )?;

        Ok(Self {
            window,
            wgpu_setup,
            simulation,
            pipeline,
            config,
            is_surface_configured: false,
            frame_count: 0,
            log_timer: std::time::Instant::now(),

            spawner,
            food_spawner,
        })
    }

    fn adjust_scout_ratio(&mut self, delta: f32) {
        let new_ratio = (self.config.scout_ratio + delta).clamp(0.0, 1.0);
        if new_ratio != self.config.scout_ratio {
            self.config.scout_ratio = new_ratio;
            // Read current ant state from GPU (includes positions/directions)
            let ants = self
                .simulation
                .read_ant_state(&self.wgpu_setup.device, &self.wgpu_setup.queue);
            // Update types while preserving positions/directions
            self.spawner.ants_mut().copy_from_slice(&ants);
            self.spawner.adjust_scout_ratio(new_ratio);
            // Write updated ants back to GPU
            self.wgpu_setup.queue.write_buffer(
                &self.simulation.ant_buffer,
                0,
                bytemuck::cast_slice(self.spawner.ants()),
            );

            // Update GPU config buffer with new scout ratio
            let gpu_config = GpuConfig::from(&self.config);
            self.wgpu_setup.queue.write_buffer(
                &self.simulation.config_buffer,
                0,
                bytemuck::bytes_of(&gpu_config),
            );
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.wgpu_setup.resize(width, height);
        self.simulation.resize(
            &self.wgpu_setup.device,
            &self.wgpu_setup.queue,
            width,
            height,
        );
        // Update food spawner's grid size and re-upload data
        self.food_spawner.food_grid = crate::food::FoodGrid::new(width, height);
        self.food_spawner
            .spawn_food_circle(self.config.food_source_radius);
        self.wgpu_setup.queue.write_buffer(
            &self.simulation.food_buffer,
            0,
            bytemuck::cast_slice(&self.food_spawner.food_grid.data),
        );
        self.pipeline
            .on_resize(&self.wgpu_setup.device, &self.simulation);
        self.is_surface_configured = true;
    }

    fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();
        self.frame_count += 1;
        if self.log_timer.elapsed().as_secs_f32() >= 1.0 {
            log::debug!("fps: {}", self.frame_count,);
            log::debug!("ratio: {:.2}", self.config.scout_ratio);
            self.frame_count = 0;
            self.log_timer = std::time::Instant::now();
        }

        if !self.is_surface_configured {
            return Ok(());
        }

        let output = match self.wgpu_setup.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                self.wgpu_setup
                    .surface
                    .configure(&self.wgpu_setup.device, &self.wgpu_setup.config);
                surface_texture
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => {
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.wgpu_setup
                    .surface
                    .configure(&self.wgpu_setup.device, &self.wgpu_setup.config);
                return Ok(());
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                anyhow::bail!("Lost device");
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.simulation
            .update(&self.wgpu_setup.device, &self.wgpu_setup.queue);

        let mut encoder =
            self.wgpu_setup
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.pipeline.background_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            self.pipeline
                .draw(&mut render_pass, self.simulation.ant_count as u32);
        }

        self.wgpu_setup
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

impl State {
    fn save_screenshot(&mut self) -> anyhow::Result<()> {
        let background_color = self.pipeline.background_color;
        let filename = save_state(
            &self.wgpu_setup.device,
            &self.wgpu_setup.queue,
            &self.wgpu_setup.config,
            &self.config,
            background_color,
        )?;

        let png_path = filename.with_extension("png");
        save_screenshot(
            &self.wgpu_setup.device,
            &self.wgpu_setup.queue,
            &self.wgpu_setup.config,
            background_color,
            &png_path,
            |encoder: &mut wgpu::CommandEncoder, texture_view: &wgpu::TextureView| {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Screenshot Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: texture_view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(background_color),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                    multiview_mask: None,
                });
                self.simulation
                    .update(&self.wgpu_setup.device, &self.wgpu_setup.queue);
                self.pipeline
                    .draw(&mut render_pass, self.simulation.ant_count as u32);
            },
        )?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct App {
    state: Option<State>,
    config: Config,
}

impl App {
    fn new(config: Config) -> Self {
        Self {
            state: None,
            config,
        }
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Gekke Mieren")
            .with_inner_size(winit::dpi::Size::Physical(winit::dpi::PhysicalSize::new(
                self.config.window_width,
                self.config.window_height,
            )));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.state = Some(pollster::block_on(State::new(window, self.config.clone())).unwrap());
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: State) {
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => match state.render() {
                Ok(_) => {}
                Err(e) => {
                    log::error!("{:?}", e);
                    event_loop.exit();
                }
            },
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match key {
                KeyCode::Escape => event_loop.exit(),
                KeyCode::ArrowUp => state.adjust_scout_ratio(state.config.ratio_step),
                KeyCode::ArrowDown => state.adjust_scout_ratio(-state.config.ratio_step),
                KeyCode::KeyS => {
                    if let Err(e) = state.save_screenshot() {
                        log::error!("Failed to save screenshot: {:?}", e);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    env_logger::init();

    // Get config file path from command line arguments
    // First argument (if any) is the config file path, otherwise use default "config.toml"
    let args: Vec<String> = std::env::args().collect();
    let config_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("config.toml")
    };

    // Load configuration from file
    let config = Config::from_file(&config_path).with_context(|| {
        format!(
            "Failed to load configuration from: {}",
            config_path.display()
        )
    })?;

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new(config);
    event_loop.run_app(&mut app)?;

    Ok(())
}

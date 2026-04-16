use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{
    color_scheme::{ColorScheme, Palette},
    config::Config,
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
    current_palette: Palette,
    current_scout_ratio: f32,
    spawner: RandomSpawner,
}

impl State {
    async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let wgpu_setup = WgpuSetup::new(window.clone()).await?;

        let config = Config::default();
        let sim_config = config.sim_config();

        let spawner = RandomSpawner::new(
            Colony::default(),
            config.n_ants,
            config.initial_scout_ratio,
            config.base_speed,
        );

        let simulation = SimulationPipeline::new(
            &wgpu_setup.device,
            wgpu_setup.config.width,
            wgpu_setup.config.height,
            sim_config,
            spawner.colony(),
            spawner.ants(),
        );

        let color_scheme = ColorScheme::from_palette(Palette::BoldHues);
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
            current_palette: Palette::BoldHues,
            current_scout_ratio: config.initial_scout_ratio,
            spawner,
        })
    }

    fn cycle_palette(&mut self) {
        self.current_palette = self.current_palette.next();
        let scheme = ColorScheme::from_palette(self.current_palette);
        self.pipeline
            .set_color_scheme(&self.wgpu_setup.queue, scheme);
    }

    fn adjust_scout_ratio(&mut self, delta: f32) {
        let new_ratio = (self.current_scout_ratio + delta).clamp(0.0, 1.0);
        if new_ratio != self.current_scout_ratio {
            self.current_scout_ratio = new_ratio;
            // Read current ant state from GPU (includes positions/directions)
            let ants = self.simulation.read_ant_state(&self.wgpu_setup.device, &self.wgpu_setup.queue);
            // Update types while preserving positions/directions
            self.spawner.ants_mut().copy_from_slice(&ants);
            self.spawner.adjust_scout_ratio(new_ratio);
            // Write updated ants back to GPU
            self.wgpu_setup.queue.write_buffer(
                &self.simulation.ant_buffer,
                0,
                bytemuck::cast_slice(self.spawner.ants()),
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
        self.pipeline
            .on_resize(&self.wgpu_setup.device, &self.simulation);
        self.is_surface_configured = true;
    }

    fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();
        self.frame_count += 1;
        if self.log_timer.elapsed().as_secs_f32() >= 1.0 {
            log::debug!("fps: {}", self.frame_count,);
            log::debug!("ratio: {:.2}", self.current_scout_ratio);
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
                self.simulation.update(&self.wgpu_setup.device, &self.wgpu_setup.queue);
                self.pipeline.draw(&mut render_pass, self.simulation.ant_count as u32);
            },
        )?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct App {
    state: Option<State>,
}

impl App {
    fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("Gekke Mieren")
            .with_inner_size(winit::dpi::Size::Physical(winit::dpi::PhysicalSize::new(
                1024, 768,
            )));
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.state = Some(pollster::block_on(State::new(window)).unwrap());
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
                KeyCode::KeyC => state.cycle_palette(),
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

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}

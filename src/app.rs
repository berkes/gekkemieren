use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{
    pheromone::SimConfig,
    pipeline::Pipeline,
    wgpu_setup::WgpuSetup,
};

const DECAY_AMOUNT: u32 = 1;
const MAX_STRENGTH: u32 = 1000;
const DEPOSIT_AMOUNT: u32 = 50;
const DOT_RADIUS: f32 = 0.001;
const COLLISION_RADIUS: f32 = 0.0001;
const COLLISION_ANGLE_MIN: f32 = 1.169_370_6; // 67deg
const COLLISION_ANGLE_MAX: f32 = 1.954_768_8; // 112deg

#[derive(Debug)]
pub struct State {
    window: Arc<Window>,
    wgpu_setup: WgpuSetup,
    pipeline: Pipeline,
    is_surface_configured: bool,
    frame_count: u32,
    fps_timer: std::time::Instant,
}

impl State {
    async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let wgpu_setup = WgpuSetup::new(window.clone()).await?;
        let sim_config = SimConfig {
            decay_amount: DECAY_AMOUNT,
            max_strength: MAX_STRENGTH,
            deposit_amount: DEPOSIT_AMOUNT,
            dot_radius: DOT_RADIUS,
            collision_radius: COLLISION_RADIUS,
            collision_angle_min: COLLISION_ANGLE_MIN,
            collision_angle_max: COLLISION_ANGLE_MAX,
            _pad: 0,
        };
        let pipeline = Pipeline::new(&wgpu_setup.device, &wgpu_setup.config, sim_config)?;

        Ok(Self {
            window,
            wgpu_setup,
            pipeline,
            is_surface_configured: false,
            frame_count: 0,
            fps_timer: std::time::Instant::now(),
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.wgpu_setup.resize(width, height);
        self.pipeline
            .resize(&self.wgpu_setup.device, &self.wgpu_setup.queue, width, height);
        self.is_surface_configured = true;
    }

    fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();
        self.frame_count += 1;
        if self.fps_timer.elapsed().as_secs_f32() >= 1.0 {
            log::debug!("fps: {}", self.frame_count);
            self.frame_count = 0;
            self.fps_timer = std::time::Instant::now();
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

        self.pipeline
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
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            self.pipeline.draw(&mut render_pass);
        }

        self.wgpu_setup
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();

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
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => event_loop.exit(),
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

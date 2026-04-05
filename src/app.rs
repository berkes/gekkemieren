use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::{
    shader::{INDICES, ShaderManager},
    texture::Texture,
    wgpu_setup::WgpuSetup,
};

#[derive(Debug)]
pub struct State {
    window: Arc<Window>,
    wgpu_setup: WgpuSetup,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    diffuse_bind_group: wgpu::BindGroup,
    #[allow(unused)]
    diffuse_texture: Texture,
    mouse_position: winit::dpi::PhysicalPosition<f64>,
    start_time: std::time::Instant,
    is_surface_configured: bool,
}

impl State {
    async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let wgpu_setup = WgpuSetup::new(window.clone()).await?;
        let shader_manager = ShaderManager::new(wgpu_setup.device.clone());
        let (vertex_buffer, index_buffer) = shader_manager.create_buffers();
        let num_indices = INDICES.len() as u32;

        let diffuse_bytes = include_bytes!("tree.jpg");
        let diffuse_texture = Texture::from_bytes(
            &wgpu_setup.device,
            &wgpu_setup.queue,
            diffuse_bytes,
            "tree.jpg",
        )?;
        let diffuse_bind_group = shader_manager.create_bind_group(&diffuse_texture);
        let render_pipeline = shader_manager.create_render_pipeline(&wgpu_setup.config);

        Ok(Self {
            window,
            wgpu_setup,
            render_pipeline,
            index_buffer,
            vertex_buffer,
            num_indices,
            diffuse_bind_group,
            diffuse_texture,

            start_time: std::time::Instant::now(),
            mouse_position: winit::dpi::PhysicalPosition::default(),
            is_surface_configured: false,
        })
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.wgpu_setup.resize(width, height);
        self.is_surface_configured = true;
    }

    fn update(&mut self) {
        // Later
    }

    fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

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

        let mut encoder =
            self.wgpu_setup
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let red = self.mouse_position.x / self.window.inner_size().width as f64;
            let green = self.mouse_position.y / self.window.inner_size().height as f64;
            let blue = (self.start_time.elapsed().as_secs_f64() * 0.5).sin() * 0.5 + 0.5;

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: red as f64,
                            g: green as f64,
                            b: blue as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.wgpu_setup
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }

    fn handle_mouse_move(
        &mut self,
        _event_loop: &ActiveEventLoop,
        position: winit::dpi::PhysicalPosition<f64>,
    ) {
        self.mouse_position = position;
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
            WindowEvent::RedrawRequested => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("{:?}", e);
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => state.handle_mouse_move(event_loop, position),
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

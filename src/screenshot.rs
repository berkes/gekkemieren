use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    sync::mpsc::sync_channel,
};

use anyhow::Result;
use chrono::Local;
use png::{BitDepth, ColorType, Encoder};
use wgpu::{Device, Queue, SurfaceConfiguration};

use crate::config::Config;

pub fn save_state(
    _device: &Device,
    _queue: &Queue,
    _config: &SurfaceConfiguration,
    config: &Config,
    _background_color: wgpu::Color,
) -> Result<PathBuf> {
    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let saves_dir = Path::new("saves");
    fs::create_dir_all(saves_dir)?;

    let filename = saves_dir.join(format!("{}_sim", timestamp));

    // Save config as TOML
    let toml_path = filename.with_extension("toml");
    let toml_string = toml::to_string_pretty(config)?;
    fs::write(&toml_path, toml_string)?;
    log::debug!("Saved simulation config to {}", toml_path.display());

    log::info!("Saved state to {}/{}", saves_dir.display(), timestamp);
    Ok(filename)
}

pub fn save_screenshot(
    device: &Device,
    queue: &Queue,
    config: &SurfaceConfiguration,
    _background_color: wgpu::Color,
    path: &Path,
    render_callback: impl FnOnce(&mut wgpu::CommandEncoder, &wgpu::TextureView),
) -> Result<()> {
    let width = config.width;
    let height = config.height;
    let format = config.format;

    let output_buffer_size = (width * height * 4) as usize;

    // Create output buffer
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Screenshot Output Buffer"),
        size: output_buffer_size as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Create screenshot texture
    let texture_extent = wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let screenshot_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Screenshot Texture"),
        size: texture_extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });

    let screenshot_view = screenshot_texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Create encoder and render
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenshot Encoder"),
    });

    render_callback(&mut encoder, &screenshot_view);

    // Copy texture to buffer
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &screenshot_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: None,
            },
        },
        texture_extent,
    );

    queue.submit(std::iter::once(encoder.finish()));

    // Map and read buffer
    let (sender, receiver) = sync_channel(1);
    let buffer_slice = output_buffer.slice(..);
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).ok();
    });

    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: None,
    });

    receiver.recv()??;

    // Encode and save PNG
    let buffer_slice = output_buffer.slice(..);
    let mapping = buffer_slice.get_mapped_range();
    let data = &mapping[0..output_buffer_size];

    let mut png_encoder = Encoder::new(File::create(path)?, width, height);
    png_encoder.set_color(ColorType::Rgba);
    png_encoder.set_depth(BitDepth::Eight);
    let mut writer = png_encoder.write_header()?;
    writer.write_image_data(data)?;

    Ok(())
}

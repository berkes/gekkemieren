use anyhow::Context as _;
use resvg::{tiny_skia, usvg};

pub const DASHBOARD_SVG: &str = include_str!("../assets/dashboard.svg");

const RENDER_SCALE: u32 = 2;
const GAUGE_RADIUS: f64 = 52.0;
const GAUGE_ARC_LENGTH: f64 = 2.0 * std::f64::consts::PI * GAUGE_RADIUS;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DashboardValues {
    pub food_home_total: u64,
    pub food_home_per_min: f64,
    pub food_on_canvas: u64,
    pub food_deposited_total: u64,
    pub deaths: u64,
    pub scout_ratio: f64,
}

/// Substitutes only value-printing and gauge placeholders; layout lives in the SVG.
pub fn fill_template(template: &str, values: &DashboardValues) -> String {
    let gauge_arc_offset = GAUGE_ARC_LENGTH * (1.0 - values.scout_ratio.clamp(0.0, 1.0));

    let replacements = [
        ("{{food_home_total}}", values.food_home_total.to_string()),
        (
            "{{food_home_per_min}}",
            format!("{:.1}", values.food_home_per_min),
        ),
        ("{{food_on_canvas}}", values.food_on_canvas.to_string()),
        (
            "{{food_deposited_total}}",
            values.food_deposited_total.to_string(),
        ),
        ("{{deaths}}", values.deaths.to_string()),
        (
            "{{scout_ratio}}",
            format!("{:.0}%", values.scout_ratio * 100.),
        ),
        ("{{scout_ratio_arc}}", format!("{:.2}", gauge_arc_offset)),
    ];

    let mut svg = template.to_string();
    for (placeholder, value) in replacements {
        svg = svg.replace(placeholder, &value);
    }
    svg
}

fn parse_svg(svg: &str) -> Option<usvg::Tree> {
    let mut options = usvg::Options::default();
    options
        .fontdb_mut()
        .load_font_data(epaint_default_fonts::HACK_REGULAR.to_vec());
    options.font_family = "Hack".to_string();
    usvg::Tree::from_str(svg, &options).ok()
}

/// Rasterizes at the SVG's own size times RENDER_SCALE; the result is premultiplied RGBA.
pub fn rasterize(svg: &str) -> Option<tiny_skia::Pixmap> {
    let tree = parse_svg(svg)?;
    let size = tree.size();
    let scale = RENDER_SCALE as f32;
    let mut pixmap = tiny_skia::Pixmap::new(
        (size.width() * scale) as u32,
        (size.height() * scale) as u32,
    )?;
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    Some(pixmap)
}

/// Fullscreen overlay drawn as a single textured quad on top of the simulation.
/// The texture holds the rasterized SVG dashboard in premultiplied RGBA.
#[derive(Debug)]
pub struct Dashboard {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    texture: wgpu::Texture,
}

impl Dashboard {
    pub fn new(device: &wgpu::Device, texture_format: wgpu::TextureFormat) -> anyhow::Result<Self> {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/dashboard.wgsl"));
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("dashboard_pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
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

        let tree = parse_svg(DASHBOARD_SVG).context("dashboard.svg must parse")?;
        let size = tree.size();
        let scale = RENDER_SCALE as f32;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("dashboard_texture"),
            size: wgpu::Extent3d {
                width: (size.width() * scale) as u32,
                height: (size.height() * scale) as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("dashboard_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("dashboard_bind_group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Ok(Self {
            pipeline,
            bind_group,
            texture,
        })
    }

    pub fn update(&mut self, queue: &wgpu::Queue, values: &DashboardValues) {
        let svg = fill_template(DASHBOARD_SVG, values);
        let Some(pixmap) = rasterize(&svg) else {
            return;
        };
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            pixmap.data(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(pixmap.width() * 4),
                rows_per_image: Some(pixmap.height()),
            },
            wgpu::Extent3d {
                width: pixmap.width(),
                height: pixmap.height(),
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn draw<'pass>(&'pass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values() -> DashboardValues {
        DashboardValues {
            food_home_total: 1234,
            food_home_per_min: 42.5,
            food_on_canvas: 987,
            food_deposited_total: 5000,
            deaths: 12,
            scout_ratio: 0.2,
        }
    }

    #[test]
    fn fill_template_replaces_all_placeholders() {
        let svg = fill_template(DASHBOARD_SVG, &values());
        assert!(!svg.contains("{{"), "unreplaced placeholder left: {}", svg);
        assert!(svg.contains("1234"));
        assert!(svg.contains("42.5"));
        assert!(svg.contains("987"));
        assert!(svg.contains("5000"));
        assert!(svg.contains("12"));
        assert!(svg.contains("20%"));
        assert!(svg.contains("width=\"1100\""));
    }

    #[test]
    fn gauge_arc_offset_tracks_scout_ratio() {
        let mut v = values();
        v.scout_ratio = 0.0;
        let svg = fill_template(DASHBOARD_SVG, &v);
        assert!(svg.contains(&format!("stroke-dashoffset=\"{:.2}\"", GAUGE_ARC_LENGTH)));

        v.scout_ratio = 1.0;
        let svg = fill_template(DASHBOARD_SVG, &v);
        assert!(svg.contains("stroke-dashoffset=\"0.00\""));
    }

    #[test]
    fn rasterize_returns_scaled_pixmap_with_transparent_background() {
        let svg = fill_template(DASHBOARD_SVG, &values());
        let pixmap = rasterize(&svg).expect("svg should rasterize");
        assert_eq!(pixmap.width(), 1100 * RENDER_SCALE);
        assert_eq!(pixmap.height(), 1100 * RENDER_SCALE);

        let corner_alpha = pixmap.pixel(0, 0).expect("corner pixel").alpha();
        assert_eq!(
            corner_alpha, 0,
            "background outside the bar must be transparent"
        );

        let bar_alpha = pixmap
            .pixel(100 * RENDER_SCALE, 960 * RENDER_SCALE)
            .expect("bar pixel")
            .alpha();
        assert!(bar_alpha > 0, "bottom bar must be painted");
    }

    #[test]
    fn rasterize_rejects_invalid_svg() {
        assert!(rasterize("<svg").is_none());
    }
}

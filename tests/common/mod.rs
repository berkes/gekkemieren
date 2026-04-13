/// Headless GPU setup for integration tests.
/// Creates a wgpu device and queue without a window or surface.
pub struct HeadlessGpuSetup {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl HeadlessGpuSetup {
    pub async fn new() -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: Default::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
            display: None,
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: None,
                // Force the software renderer (lavapipe on Linux) so that
                // floating-point results are identical across all machines.
                force_fallback_adapter: true,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        Ok(Self { device, queue })
    }
}

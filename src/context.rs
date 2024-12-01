use std::sync::Arc;



pub struct WgpuContext {
    pub event_loop: Option<winit::event_loop::EventLoop<()>>,
    pub window: winit::window::Window,
    pub device: Arc<wgpu::Device>,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub surface_config: wgpu::SurfaceConfiguration,
}


fn init_window(
    size: winit::dpi::Size,
    event_loop: &winit::event_loop::EventLoop<()>,
) -> Result<winit::window::Window, Box<dyn std::error::Error>> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(size)
        .build(event_loop)?;
    Ok(window)
}

pub async fn init_wgpu(width: u32, height: u32, _bind_id: &str) -> Result<WgpuContext, String> {
    let event_loop = winit::event_loop::EventLoop::new().map_err(|e| e.to_string())?;

    let window = init_window(
        winit::dpi::Size::Physical(winit::dpi::PhysicalSize::new(width, height)),
        &event_loop,
    )
    .map_err(|e| e.to_string())?;

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        flags: wgpu::InstanceFlags::default(),
        gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    });

    let surface = unsafe {
        instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&window).unwrap())
    }
    .map_err(|e| e.to_string())?;

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .ok_or("unable to create adapter")?;

    log::info!("adapter.features = {:#?}", adapter.features());
    log::info!("adapter.limits = {:#?}", adapter.limits());

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("GPU Device"),
                required_features: adapter.features(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .map_err(|e| e.to_string())?;

    let surface_format = preferred_framebuffer_format(&surface.get_capabilities(&adapter).formats);
    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width,
        height,
        present_mode: wgpu::PresentMode::Fifo, // vsync
        alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        view_formats: vec![
            surface_format.add_srgb_suffix(),
            surface_format.remove_srgb_suffix(),
        ],
        desired_maximum_frame_latency: 1,
    };
    surface.configure(&device, &surface_config);

    Ok(WgpuContext {
        event_loop: Some(event_loop),
        window,
        device: Arc::new(device),
        queue,
        surface,
        surface_config,
    })
}

fn preferred_framebuffer_format(formats: &[wgpu::TextureFormat]) -> wgpu::TextureFormat {
    for &format in formats {
        if matches!(
            format,
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm
        ) {
            return format;
        }
    }
    formats[0]
}

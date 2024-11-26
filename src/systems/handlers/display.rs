use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};
use winit_input_helper::WinitInputHelper;

use crate::systems::RgbColor;

/// Handler for the display.
pub struct Display {
    surface: wgpu::Surface<'static>,
    queue: wgpu::Queue,
    device: wgpu::Device,
    config: wgpu::SurfaceConfiguration,

    size: PhysicalSize<u32>,
    clear_color: RgbColor,

    // This is needed because surface points to the window
    #[allow(dead_code)]
    window: Arc<Window>,
}

impl Display {
    pub async fn new(window: Arc<Window>, clear_color: RgbColor) -> Self {
        let size = window.inner_size();

        log::debug!("Creating wgpu instance");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: match cfg!(target_arch = "wasm32") {
                true => wgpu::Backends::BROWSER_WEBGPU,
                false => wgpu::Backends::PRIMARY,
            },
            ..Default::default()
        });

        log::debug!("Creating window surface");
        let surface = instance.create_surface(window.clone()).unwrap();

        log::debug!("Requesting adapter");
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("request adapter");

        log::debug!("Requesting device");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: adapter.limits(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        log::debug!("Configuring surface");
        surface.configure(&device, &config);

        log::info!("Display handler initialized");

        Self {
            surface,
            device,
            queue,
            config,

            size,
            clear_color,

            window,
        }
    }

    pub fn surface(&self) -> &wgpu::Surface {
        &self.surface
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn config(&self) -> &wgpu::SurfaceConfiguration {
        &self.config
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.size = size;
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.size.width as f32 / self.size.height as f32
    }

    pub fn update(&mut self, input: &WinitInputHelper) {
        if let Some(size) = input.window_resized() {
            self.resize(size);
        }
    }

    pub fn render(&mut self, render: impl FnOnce(&mut Display, &mut wgpu::RenderPass)) {
        let texture = self.surface.get_current_texture().unwrap();
        let texture_view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.clear_color.r() as f64,
                            g: self.clear_color.g() as f64,
                            b: self.clear_color.b() as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render(self, &mut render_pass);
        }

        // Submit render pass
        self.queue.submit(std::iter::once(encoder.finish()));
        self.device.poll(wgpu::Maintain::Wait);
        texture.present();
    }
}

/// Builder of [`Display`].
pub struct DisplayBuilder<T> {
    window: T,
    clear_color: RgbColor,
}

pub mod builder {
    use super::*;

    pub struct NoWindow;
    pub struct WithWindow(pub Arc<Window>);
}

impl DisplayBuilder<builder::NoWindow> {
    pub fn new() -> Self {
        Self {
            window: builder::NoWindow,
            clear_color: RgbColor::BLACK,
        }
    }
}

impl<T> DisplayBuilder<T> {
    pub fn with_window(self, window: Arc<Window>) -> DisplayBuilder<builder::WithWindow> {
        DisplayBuilder {
            window: builder::WithWindow(window),
            clear_color: self.clear_color,
        }
    }

    pub fn with_clear_color(mut self, clear_color: RgbColor) -> Self {
        self.clear_color = clear_color;
        self
    }
}

impl DisplayBuilder<builder::WithWindow> {
    pub async fn build(self) -> Display {
        Display::new(self.window.0, self.clear_color).await
    }
}

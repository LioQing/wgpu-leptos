use glam::*;
use wgpu::util::DeviceExt;

use crate::systems::{RgbColor, Transform};

/// Handler for the spinning pyramid.
pub struct Pyramid {
    transform: PyramidTransform,
    model: PyramidModel,

    transform_buffer: wgpu::Buffer,
    model_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,

    transform_bind_group: wgpu::BindGroup,
}

impl Pyramid {
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        transform: Transform,
        auto_rotation_speed: f32,
        height: f32,
        base_radius: f32,
        side_count: usize,
    ) -> Self {
        let transform = PyramidTransform::new(transform, auto_rotation_speed);
        let model = PyramidModel::new(height, base_radius, side_count);
        let indices = model.indices().collect::<Vec<_>>();

        log::debug!("Creating pyramid transform buffer");
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pyramid Transform Buffer"),
            contents: transform.buffer().as_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        log::debug!("Creating pyramid transform bind group layout");
        let transform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Pyramid Transform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        log::debug!("Creating pyramid transform bind group");
        let transform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Pyramid Transform Bind Group"),
            layout: &transform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transform_buffer.as_entire_binding(),
            }],
        });

        log::debug!("Creating pyramid model buffer");
        let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pyramid Model Buffer"),
            contents: model.buffer().as_bytes(),
            usage: wgpu::BufferUsages::VERTEX,
        });

        log::debug!("Creating pyramid index buffer: {indices:?}");
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pyramid Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        log::debug!("Creating pyramid shader");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Pyramid Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/pyramid.wgsl").into()),
        });

        log::debug!("Creating pyramid pipeline layout");
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pyramid Pipeline Layout"),
            bind_group_layouts: &[camera_bind_group_layout, &transform_bind_group_layout],
            push_constant_ranges: &[],
        });

        log::debug!("Creating pyramid render pipeline");
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Pyramid Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                buffers: &[PyramidVertex::BUFFER_LAYOUT],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        log::info!("Pyramid handler initialized");

        Self {
            transform,
            model,

            transform_buffer,
            model_buffer,
            index_buffer,
            render_pipeline,

            transform_bind_group,
        }
    }

    pub fn update(&mut self, dt: f32, queue: &wgpu::Queue) {
        self.transform.transform.rotate(Quat::from_axis_angle(
            Vec3::Y,
            self.transform.auto_rotation_speed * dt,
        ));

        queue.write_buffer(
            &self.transform_buffer,
            0,
            self.transform.buffer().as_bytes(),
        );
    }

    pub fn render(&self, render_pass: &mut wgpu::RenderPass, camera_bind_group: &wgpu::BindGroup) {
        let model_buffer_len =
            (std::mem::size_of::<PyramidVertex>() * (self.model.side_count + 1)) as u64;
        let index_buffer_len = (std::mem::size_of::<u16>() * self.model.side_count * 3) as u64;

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_bind_group(1, &self.transform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.model_buffer.slice(..model_buffer_len));
        render_pass.set_index_buffer(
            self.index_buffer.slice(..index_buffer_len),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..self.model.side_count as u32 * 3, 0, 0..1);
    }
}

#[derive(Debug, Clone)]
pub struct PyramidTransform {
    pub transform: Transform,
    pub auto_rotation_speed: f32,
}

impl PyramidTransform {
    pub fn new(transform: Transform, auto_rotation_speed: f32) -> Self {
        Self {
            transform,
            auto_rotation_speed,
        }
    }

    fn buffer(&self) -> PyramidTransformBuffer {
        PyramidTransformBuffer {
            transform: self.transform.matrix(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PyramidTransformBuffer {
    transform: Mat4,
}

impl PyramidTransformBuffer {
    fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(std::slice::from_ref(self))
    }
}

#[derive(Debug, Clone)]
pub struct PyramidModel {
    pub height: f32,
    pub base_radius: f32,
    pub side_count: usize,
}

impl PyramidModel {
    pub fn new(height: f32, base_radius: f32, side_count: usize) -> Self {
        Self {
            height,
            base_radius,
            side_count,
        }
    }

    pub fn indices(&self) -> impl Iterator<Item = u16> + '_ {
        (0..self.side_count).flat_map(|i| {
            [
                0,
                ((i + 1) % self.side_count + 1) as u16,
                (i % self.side_count + 1) as u16,
            ]
        })
    }

    fn buffer(&self) -> PyramidModelBuffer {
        PyramidModelBuffer::new(self.height, self.base_radius, self.side_count)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PyramidModelBuffer {
    top: PyramidVertex,
    bases: [PyramidVertex; PyramidModelBuffer::MAX_SIDES],
}

impl PyramidModelBuffer {
    const MAX_SIDES: usize = 64;

    fn new(height: f32, base_radius: f32, side_count: usize) -> Self {
        Self {
            top: PyramidVertex {
                position: vec3(0.0, height, 0.0),
                color: RgbColor::WHITE,
            },
            bases: std::array::from_fn(|i| {
                if i >= side_count {
                    return PyramidVertex {
                        position: Vec3::ZERO,
                        color: RgbColor::BLACK,
                    };
                }

                let factor = i as f32 / side_count as f32;
                let angle = factor * 2.0 * std::f32::consts::PI;
                PyramidVertex {
                    position: vec3(base_radius * angle.cos(), 0.0, base_radius * angle.sin()),
                    color: RgbColor::from_hue(factor).expect("valid color"),
                }
            }),
        }
    }

    fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(std::slice::from_ref(self))
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PyramidVertex {
    position: Vec3,
    color: RgbColor,
}

impl PyramidVertex {
    const BUFFER_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<PyramidVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                format: wgpu::VertexFormat::Float32x3,
                shader_location: 0,
            },
            wgpu::VertexAttribute {
                offset: std::mem::size_of::<Vec3>() as wgpu::BufferAddress,
                format: wgpu::VertexFormat::Float32x3,
                shader_location: 1,
            },
        ],
    };
}

/// Builder of [`Pyramid`].
pub struct PyramidBuilder<T, U, V> {
    device: T,
    surface_config: U,
    camera_bind_group_layout: V,
    transform: Transform,
    auto_rotation_speed: f32,
    height: f32,
    base_radius: f32,
    side_count: usize,
}

pub mod builder {
    pub struct NoDevice;
    pub struct WithDevice<'a>(pub &'a wgpu::Device);

    pub struct NoSurfaceConfig;
    pub struct WithSurfaceConfig<'a>(pub &'a wgpu::SurfaceConfiguration);

    pub struct NoCameraBindGroupLayout;
    pub struct WithCameraBindGroupLayout<'a>(pub &'a wgpu::BindGroupLayout);
}

impl PyramidBuilder<builder::NoDevice, builder::NoSurfaceConfig, builder::NoCameraBindGroupLayout> {
    pub fn new() -> Self {
        Self {
            device: builder::NoDevice,
            surface_config: builder::NoSurfaceConfig,
            camera_bind_group_layout: builder::NoCameraBindGroupLayout,
            transform: Transform::IDENTITY,
            auto_rotation_speed: 1.0,
            height: 1.0,
            base_radius: 1.0,
            side_count: 4,
        }
    }
}

impl<T, U, V> PyramidBuilder<T, U, V> {
    pub fn with_device(self, device: &wgpu::Device) -> PyramidBuilder<builder::WithDevice, U, V> {
        PyramidBuilder {
            device: builder::WithDevice(device),
            surface_config: self.surface_config,
            camera_bind_group_layout: self.camera_bind_group_layout,
            transform: self.transform,
            auto_rotation_speed: self.auto_rotation_speed,
            height: self.height,
            base_radius: self.base_radius,
            side_count: self.side_count,
        }
    }

    pub fn with_surface_config(
        self,
        surface_config: &wgpu::SurfaceConfiguration,
    ) -> PyramidBuilder<T, builder::WithSurfaceConfig, V> {
        PyramidBuilder {
            device: self.device,
            surface_config: builder::WithSurfaceConfig(surface_config),
            camera_bind_group_layout: self.camera_bind_group_layout,
            transform: self.transform,
            auto_rotation_speed: self.auto_rotation_speed,
            height: self.height,
            base_radius: self.base_radius,
            side_count: self.side_count,
        }
    }

    pub fn with_camera_bind_group_layout(
        self,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> PyramidBuilder<T, U, builder::WithCameraBindGroupLayout> {
        PyramidBuilder {
            device: self.device,
            surface_config: self.surface_config,
            camera_bind_group_layout: builder::WithCameraBindGroupLayout(camera_bind_group_layout),
            transform: self.transform,
            auto_rotation_speed: self.auto_rotation_speed,
            height: self.height,
            base_radius: self.base_radius,
            side_count: self.side_count,
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn with_position(mut self, position: Vec3) -> Self {
        self.transform.position = position;
        self
    }

    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.transform.rotation = rotation;
        self
    }

    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.transform.scale = scale;
        self
    }

    pub fn with_auto_rotation_speed(mut self, auto_rotation_speed: f32) -> Self {
        self.auto_rotation_speed = auto_rotation_speed;
        self
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn with_base_radius(mut self, base_radius: f32) -> Self {
        self.base_radius = base_radius;
        self
    }

    pub fn with_side_count(mut self, side_count: usize) -> Self {
        self.side_count = side_count;
        self
    }
}

impl<'a>
    PyramidBuilder<
        builder::WithDevice<'a>,
        builder::WithSurfaceConfig<'a>,
        builder::WithCameraBindGroupLayout<'a>,
    >
{
    pub fn build(self) -> Pyramid {
        Pyramid::new(
            self.device.0,
            self.surface_config.0,
            self.camera_bind_group_layout.0,
            self.transform,
            self.auto_rotation_speed,
            self.height,
            self.base_radius,
            self.side_count,
        )
    }
}

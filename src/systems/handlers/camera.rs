use glam::*;
use wgpu::util::DeviceExt;
use winit::keyboard::KeyCode;
use winit_input_helper::WinitInputHelper;

/// Handler for the camera.
pub struct Camera {
    model: CameraModel,

    model_buffer: wgpu::Buffer,

    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl Camera {
    pub const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 1e-6;

    pub fn new(
        device: &wgpu::Device,
        aspect_ratio: f32,
        position: Vec3,
        pitch: f32,
        yaw: f32,
        vertical_fov: f32,
        z_near: f32,
        z_far: f32,
        speed: f32,
        mouse_sensitivity: f32,
    ) -> Self {
        let model = CameraModel::new(
            position,
            pitch,
            yaw,
            vertical_fov,
            z_near,
            z_far,
            speed,
            mouse_sensitivity,
        );

        log::debug!("Creating camera model buffer");
        let model_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Model Buffer"),
            contents: model.buffer(aspect_ratio).as_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        log::debug!("Creating camera model bind group layout");
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Model Bind Group Layout"),
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

        log::debug!("Creating camera model bind group");
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Model Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: model_buffer.as_entire_binding(),
            }],
        });

        Self {
            model,

            model_buffer,

            bind_group_layout,
            bind_group,
        }
    }

    /// Camera bind group layout.
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    /// Camera bind group.
    ///
    /// A single [`Mat4`] buffer bind group.
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn model(&self) -> &CameraModel {
        &self.model
    }

    pub fn update(
        &mut self,
        dt: f32,
        queue: &wgpu::Queue,
        aspect_ratio: f32,
        input: &WinitInputHelper,
    ) {
        let mut updated = false;

        let right = self.model.right();
        let forward = (self.model.forward() * (Vec3::ONE - CameraModel::UP)).normalize();

        // Movement
        if input.key_held(KeyCode::KeyW) {
            self.model.position += forward * self.model.speed * dt;
            updated = true;
        } else if input.key_held(KeyCode::KeyS) {
            self.model.position -= forward * self.model.speed * dt;
            updated = true;
        }

        if input.key_held(KeyCode::KeyA) {
            self.model.position -= right * self.model.speed * dt;
            updated = true;
        } else if input.key_held(KeyCode::KeyD) {
            self.model.position += right * self.model.speed * dt;
            updated = true;
        }

        if input.key_held(KeyCode::Space) {
            self.model.position += CameraModel::UP * self.model.speed * dt;
            updated = true;
        } else if input.key_held(KeyCode::ShiftLeft) {
            self.model.position -= CameraModel::UP * self.model.speed * dt;
            updated = true;
        }

        // Rotation
        if input.mouse_diff() != (0.0, 0.0) {
            let pitch_delta = input.mouse_diff().1.to_radians() * self.model.mouse_sensitivity;
            let yaw_delta = input.mouse_diff().0.to_radians() * self.model.mouse_sensitivity;

            self.model.pitch =
                (self.model.pitch - pitch_delta).clamp(-Self::PITCH_LIMIT, Self::PITCH_LIMIT);
            self.model.yaw = (self.model.yaw - yaw_delta).rem_euclid(2.0 * std::f32::consts::PI);

            updated = true;
        }

        // Update buffer
        if updated || input.window_resized().is_some() {
            queue.write_buffer(
                &self.model_buffer,
                0,
                self.model.buffer(aspect_ratio).as_bytes(),
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct CameraModel {
    pub position: Vec3,
    pub pitch: f32,
    pub yaw: f32,
    pub vertical_fov: f32,
    pub z_near: f32,
    pub z_far: f32,
    pub speed: f32,
    pub mouse_sensitivity: f32,
}

impl CameraModel {
    const FORWARD: Vec3 = Vec3::NEG_Z;
    const UP: Vec3 = Vec3::Y;

    pub fn new(
        position: Vec3,
        pitch: f32,
        yaw: f32,
        vertical_fov: f32,
        z_near: f32,
        z_far: f32,
        speed: f32,
        mouse_sensitivity: f32,
    ) -> Self {
        Self {
            position,
            pitch,
            yaw,
            vertical_fov,
            z_near,
            z_far,
            speed,
            mouse_sensitivity,
        }
    }

    pub fn forward(&self) -> Vec3 {
        Quat::from_euler(EulerRot::ZYX, 0.0, self.yaw, self.pitch) * Self::FORWARD
    }

    pub fn right(&self) -> Vec3 {
        self.forward().cross(Self::UP).normalize()
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_to_rh(self.position, self.forward(), Self::UP)
    }

    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh(self.vertical_fov, aspect_ratio, self.z_near, self.z_far)
    }

    fn buffer(&self, aspect_ratio: f32) -> CameraModelBuffer {
        CameraModelBuffer::new(self.projection_matrix(aspect_ratio) * self.view_matrix())
    }
}

/// Camera model buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraModelBuffer {
    view_projection: Mat4,
}

impl CameraModelBuffer {
    fn new(view_projection: Mat4) -> Self {
        Self { view_projection }
    }

    fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

/// Builder of [`Camera`].
pub struct CameraBuilder<T, U> {
    device: T,
    aspect_ratio: U,
    position: Vec3,
    pitch: f32,
    yaw: f32,
    vertical_fov: f32,
    z_near: f32,
    z_far: f32,
    speed: f32,
    mouse_sensitivity: f32,
}

pub mod builder {
    pub struct NoDevice;
    pub struct WithDevice<'a>(pub &'a wgpu::Device);

    pub struct NoAspectRatio;
    pub struct WithAspectRatio(pub f32);
}

impl CameraBuilder<builder::NoDevice, builder::NoAspectRatio> {
    pub fn new() -> Self {
        Self {
            device: builder::NoDevice,
            aspect_ratio: builder::NoAspectRatio,
            position: Vec3::ZERO,
            pitch: 0.0,
            yaw: 0.0,
            vertical_fov: 60f32.to_radians(),
            z_near: 1e-3,
            z_far: 1e3,
            speed: 1.0,
            mouse_sensitivity: 0.1,
        }
    }
}

impl<T, U> CameraBuilder<T, U> {
    pub fn with_device(self, device: &wgpu::Device) -> CameraBuilder<builder::WithDevice, U> {
        CameraBuilder {
            device: builder::WithDevice(device),
            aspect_ratio: self.aspect_ratio,
            position: self.position,
            pitch: self.pitch,
            yaw: self.yaw,
            vertical_fov: self.vertical_fov,
            z_near: self.z_near,
            z_far: self.z_far,
            speed: self.speed,
            mouse_sensitivity: self.mouse_sensitivity,
        }
    }

    pub fn with_aspect_ratio(
        self,
        aspect_ratio: f32,
    ) -> CameraBuilder<T, builder::WithAspectRatio> {
        CameraBuilder {
            device: self.device,
            aspect_ratio: builder::WithAspectRatio(aspect_ratio),
            position: self.position,
            pitch: self.pitch,
            yaw: self.yaw,
            vertical_fov: self.vertical_fov,
            z_near: self.z_near,
            z_far: self.z_far,
            speed: self.speed,
            mouse_sensitivity: self.mouse_sensitivity,
        }
    }

    pub fn with_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    pub fn with_pitch(mut self, pitch: f32) -> Self {
        self.pitch = pitch;
        self
    }

    pub fn with_yaw(mut self, yaw: f32) -> Self {
        self.yaw = yaw;
        self
    }

    pub fn with_vertical_fov(mut self, vertical_fov: f32) -> Self {
        self.vertical_fov = vertical_fov;
        self
    }

    pub fn with_z_near(mut self, z_near: f32) -> Self {
        self.z_near = z_near;
        self
    }

    pub fn with_z_far(mut self, z_far: f32) -> Self {
        self.z_far = z_far;
        self
    }

    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub fn with_mouse_sensitivity(mut self, mouse_sensitivity: f32) -> Self {
        self.mouse_sensitivity = mouse_sensitivity;
        self
    }
}

impl<'a> CameraBuilder<builder::WithDevice<'a>, builder::WithAspectRatio> {
    pub fn build(self) -> Camera {
        Camera::new(
            self.device.0,
            self.aspect_ratio.0,
            self.position,
            self.pitch,
            self.yaw,
            self.vertical_fov,
            self.z_near,
            self.z_far,
            self.speed,
            self.mouse_sensitivity,
        )
    }
}

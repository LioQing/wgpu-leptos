use std::sync::Arc;

use glam::*;
use winit::window::Window;

use crate::{
    engine,
    systems::{handlers, Args, ExternalSignal, RgbColor},
};

/// Pipeline.
pub struct Pipeline {
    time: handlers::Time,
    display: handlers::Display,
    cursor_lock: handlers::CursorLock,
    camera: handlers::Camera,
    pyramid: handlers::Pyramid,
}

impl engine::SystemPipeline for Pipeline {
    type Args = Args;
    type ExternalSignal = ExternalSignal;

    async fn init(window: Arc<Window>, configs: Self::Args) -> Self {
        log::debug!("Initializing system pipeline");

        let time = handlers::TimeBuilder::new()
            .with_fps_limit(configs.fps_limit)
            .build();
        let display = handlers::DisplayBuilder::new()
            .with_window(window.clone())
            .with_clear_color(RgbColor::DARK_SLATE_GRAY)
            .build()
            .await;
        let cursor_lock = handlers::CursorLockBuilder::new()
            .with_window(window.clone())
            .with_should_lock_cursor(true)
            .build();
        let camera = handlers::CameraBuilder::new()
            .with_device(display.device())
            .with_aspect_ratio(display.aspect_ratio())
            .with_position(vec3(0.0, 0.5, 2.0))
            .build();
        let pyramid = handlers::PyramidBuilder::new()
            .with_device(display.device())
            .with_surface_config(display.config())
            .with_camera_bind_group_layout(camera.bind_group_layout())
            .build();

        log::info!("System pipeline initialized");

        Self {
            time,
            display,
            cursor_lock,
            camera,
            pyramid,
        }
    }

    fn window_event(&mut self, _: &mut engine::Items, event: &winit::event::WindowEvent) {
        self.cursor_lock.window_event(event);
    }

    fn update(&mut self, items: &mut engine::Items) {
        self.time.update();
        self.display.update(&items.input);
        self.cursor_lock.update(&mut items.input);
        self.pyramid.update(self.time.delta(), self.display.queue());

        if self.cursor_lock.is_cursor_locked() {
            self.camera.update(
                self.time.delta(),
                self.display.queue(),
                self.display.aspect_ratio(),
                &items.input,
            );
        }

        self.display
            .render(|pass| self.pyramid.render(pass, self.camera.bind_group()));

        self.time.end_frame(items.window.clone());
    }

    fn external_signal(&mut self, items: &mut engine::Items, signal: Self::ExternalSignal) {
        match signal {
            ExternalSignal::Resize(resize) => {
                let _ = items
                    .window
                    .request_inner_size(winit::dpi::PhysicalSize::new(resize.width, resize.height));
            }
        }
    }
}

use std::sync::Arc;

use winit::{dpi::LogicalSize, window::Window};

use crate::{
    engine,
    systems::{handlers, Args, InSignal},
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
    type InSignal = InSignal;

    async fn init(window: Arc<Window>, configs: Self::Args) -> Self {
        log::debug!("Initializing system pipeline");

        let time = handlers::TimeBuilder::new()
            .with_fps_limit(configs.fps_limit)
            .build();
        let display = handlers::DisplayBuilder::new()
            .with_window(window.clone())
            .with_clear_color(configs.clear_color)
            .build()
            .await;
        let cursor_lock = handlers::CursorLockBuilder::new()
            .with_window(window.clone())
            .with_should_lock_cursor(true)
            .build();
        let camera = handlers::CameraBuilder::new()
            .with_device(display.device())
            .with_aspect_ratio(display.aspect_ratio())
            .build();
        let pyramid = handlers::PyramidBuilder::new()
            .with_device(display.device())
            .with_surface_config(display.config())
            .with_camera_bind_group_layout(camera.bind_group_layout())
            .with_pyramid_transform(configs.pyramid_transform)
            .with_model(configs.pyramid_model)
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
        self.pyramid.update(self.time.delta());

        if self.cursor_lock.is_cursor_locked() || items.input.window_resized().is_some() {
            self.camera.update(
                self.time.delta(),
                self.display.queue(),
                self.display.aspect_ratio(),
                &items.input,
            );
        }

        self.display.render(|display, pass| {
            self.pyramid
                .render(display.queue(), pass, self.camera.bind_group())
        });

        self.time.end_frame(items.window.clone());
    }

    fn in_signal(&mut self, items: &mut engine::Items, signal: Self::InSignal) {
        match signal {
            InSignal::Resize(resize) => {
                log::debug!(
                    "Resize incoming signal: {} x {}",
                    resize.width,
                    resize.height
                );
                let _ = items
                    .window
                    .request_inner_size(LogicalSize::new(resize.width, resize.height));
            }
            InSignal::PyramidTransformUpdate(update) => {
                log::debug!("Pyramid transform incoming signal");
                self.pyramid.set_transform(update.transform);
            }
            InSignal::PyramidModelUpdate(update) => {
                log::debug!("Pyramid model incoming signal");
                self.pyramid.set_model(update.model);
            }
        }
    }
}

use std::sync::Arc;

use chrono::prelude::*;
use winit::window::Window;

use crate::{engine::utils, systems::FpsLimit};

/// Handler for time-related operations.
pub struct Time {
    fps_limit: FpsLimit,
    delta: f32,
    frame_timer: DateTime<Utc>,
    start_timer: DateTime<Utc>,
}

impl Time {
    pub fn new(fps_limit: FpsLimit) -> Self {
        Self {
            fps_limit,
            delta: 0.0,
            frame_timer: Utc::now(),
            start_timer: Utc::now(),
        }
    }

    pub fn update(&mut self) {
        // Calculate delta time
        self.delta = self.time_since_last_frame();

        // Update frame timer
        self.frame_timer = Utc::now();
    }

    pub fn end_frame(&mut self, window: Arc<Window>) {
        let since_last = self.time_since_last_frame();

        // Limit the frame rate
        match self.fps_limit.as_secs_f32() {
            Some(secs) if since_last < secs => Self::set_timeout_redraw(window, secs - since_last),
            _ => window.request_redraw(),
        }
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }

    pub fn elapsed(&self) -> f32 {
        Utc::now()
            .signed_duration_since(self.start_timer)
            .num_nanoseconds()
            .expect("nanoseconds since start") as f32
            * 1e-9
    }

    pub fn time_since_last_frame(&self) -> f32 {
        Utc::now()
            .signed_duration_since(self.frame_timer)
            .num_nanoseconds()
            .expect("nanoseconds since last frame") as f32
            * 1e-9
    }

    fn set_timeout_redraw(window: Arc<Window>, duration: f32) {
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                utils::set_timeout(move || {
                    window.request_redraw();
                }, (duration * 1000.0) as i32)
            } else {
                std::thread::sleep(std::time::Duration::from_secs_f32(duration));
                window.request_redraw();
            }
        }
    }
}

/// Builder of [`Time`].
pub struct TimeBuilder {
    fps_limit: FpsLimit,
}

impl TimeBuilder {
    pub fn new() -> Self {
        Self {
            fps_limit: FpsLimit::unlimited(),
        }
    }
}

impl TimeBuilder {
    pub fn with_fps_limit(mut self, fps_limit: FpsLimit) -> Self {
        self.fps_limit = fps_limit;
        self
    }

    pub fn build(self) -> Time {
        Time::new(self.fps_limit)
    }
}

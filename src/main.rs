mod engine;
mod systems;
#[cfg(target_arch = "wasm32")]
mod ui;

pub fn main() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use leptos::*;

            console_error_panic_hook::set_once();
            console_log::init_with_level(log::Level::Debug).unwrap();

            mount_to_body(ui::App);
        } else {
            use winit::{
                dpi::LogicalSize,
                window::Window,
            };

            if std::env::var("RUST_LOG").is_err() {
                std::env::set_var("RUST_LOG", "debug");
            }
            env_logger::init();

            engine::Runner::new()
                .with_window_attributes(Window::default_attributes()
                    .with_title("wgpu")
                    .with_inner_size(LogicalSize::new(800.0, 600.0))
                )
                .with_system_pipeline::<systems::Pipeline>(systems::Args {
                    fps_limit: systems::FpsLimit::new(60),
                })
                .run()
                .unwrap();
        }
    }
}

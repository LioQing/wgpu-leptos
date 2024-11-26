use std::sync::mpsc;

use leptos::*;
use winit::{dpi::LogicalSize, platform::web::WindowAttributesExtWebSys, window::Window};

use crate::{engine, systems};

/// Engine canvas.
///
/// This component handles the creation, destruction, and restarting of the engine.
#[component]
pub fn EngineCanvas(
    #[prop(default = 100.0)] width: f64,
    #[prop(default = 100.0)] height: f64,
    #[prop(default = systems::FpsLimit::new(60))] fps_limit: systems::FpsLimit,
    #[prop(default = "wgpu + Leptos".to_string(), into)] title: String,
    #[prop(optional)] tx: Option<RwSignal<Option<mpsc::Sender<systems::EngineExternalSignal>>>>,
) -> impl IntoView {
    let node = create_node_ref::<html::Canvas>();

    let tx = tx.unwrap_or(create_rw_signal(None));

    // Cleanup the engine when the component is destroyed.
    on_cleanup(move || {
        tx.with(move |tx| {
            if let Some(tx) = tx {
                tx.send(engine::ExternalSignal::Stop).unwrap();
            }
        });
    });

    // Handle the creation of the engine.
    create_effect(move |_| {
        // We can only create the engine if the canvas node is available.
        let node = match node.get() {
            Some(node) => node,
            None => return,
        };

        let canvas = <web_sys::HtmlCanvasElement as Clone>::clone(&node);

        let window_attributes = Window::default_attributes()
            .with_canvas(Some(canvas))
            .with_inner_size(LogicalSize::new(width, height))
            .with_title(title.clone());

        let system_pipeline_args = systems::Args { fps_limit };

        // Either create a new engine or restart the existing one.
        match tx.get() {
            Some(tx) => {
                log::debug!("Restarting engine canvas");

                tx.send(engine::ExternalSignal::Start {
                    window_attributes,
                    system_pipeline_args,
                })
                .unwrap();
            }
            None => spawn_local(async move {
                log::debug!("Starting engine canvas");

                let (new_tx, rx) = mpsc::channel();
                tx.set(Some(new_tx));

                engine::Runner::new()
                    .with_window_attributes(window_attributes)
                    .with_external_signal_rx(rx)
                    .with_system_pipeline::<systems::Pipeline>(system_pipeline_args)
                    .run()
                    .unwrap();
            }),
        }
    });

    view! {
        <canvas ref=node />
    }
}

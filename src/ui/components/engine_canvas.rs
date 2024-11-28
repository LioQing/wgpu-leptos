use std::sync::mpsc;

use leptos::*;
use winit::{
    platform::web::WindowAttributesExtWebSys,
    window::{Window, WindowAttributes},
};

use crate::{engine, systems};

/// Engine canvas.
///
/// This component handles the creation, destruction, and restarting of the engine.
///
/// The `window_attributes` and `system_pipeline_args` props are only for initialization,
/// changing them after the engine is started will have no effect.
#[component]
pub fn EngineCanvas(
    #[prop(default = (move || Window::default_attributes()).into(), into)]
    window_attributes: Signal<WindowAttributes>,
    #[prop(default = (move || systems::Args::default()).into(), into)] system_pipeline_args: Signal<
        systems::Args,
    >,
    #[prop(optional, into)] tx: Option<(ReadSignal<EngineTx>, WriteSignal<EngineTx>)>,
) -> impl IntoView {
    let node = create_node_ref::<html::Canvas>();

    let (tx, set_tx) = tx.unwrap_or(create_signal(None));

    // Cleanup the engine when the component is destroyed.
    on_cleanup(move || {
        tx.with(move |tx| {
            if let Some(tx) = tx {
                tx.send(engine::InSignal::Stop).unwrap();
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

        let window_attributes = window_attributes.get_untracked().with_canvas(Some(canvas));
        let system_pipeline_args = system_pipeline_args.get_untracked();

        // Either create a new engine or restart the existing one.
        match tx.get_untracked() {
            Some(tx) => {
                log::debug!("Restarting engine canvas");

                tx.send(engine::InSignal::Start {
                    window_attributes,
                    system_pipeline_args,
                })
                .unwrap();
            }
            None => spawn_local(async move {
                log::debug!("Starting engine canvas");

                let (new_tx, rx) = mpsc::channel();
                set_tx.set(Some(new_tx));

                engine::Runner::new()
                    .with_window_attributes(window_attributes)
                    .with_rx(rx)
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

/// Engine incoming signal sender.
pub type EngineTx = Option<mpsc::Sender<systems::EngineInSignal>>;

use std::sync::mpsc;

use leptos::*;
use leptos_use::use_element_size;

use crate::{systems, ui::components::EngineCanvas};

/// Engine component.
#[component]
pub fn Engine(
    #[prop(optional, into)] running: Option<Signal<bool>>,
    #[prop(default = "".to_string(), into)] style: String,
) -> impl IntoView {
    let container_node = create_node_ref::<html::Div>();
    let container_size = use_element_size(container_node);

    let engine_tx = create_rw_signal::<Option<mpsc::Sender<systems::EngineExternalSignal>>>(None);

    // Keep the engine same size as the container.
    create_effect(move |_| {
        running.map(|running| running.get());
        container_size.width.get();
        container_size.height.get();

        // We can only resize the engine if the container node and engine tx are available.
        let (container_node, tx) = match (container_node.get(), engine_tx.get()) {
            (Some(container_node), Some(tx)) => (container_node, tx),
            _ => return,
        };

        let container = container_node.get_bounding_client_rect();
        let width = container.width();
        let height = container.height();

        log::debug!("Resizing engine to {width:.2} x {height:.2}");
        tx.send(systems::ResizeSignal::queued(width, height))
            .unwrap();
    });
    view! {
        <div ref=container_node style=style>
            <Show
                when=move || running.map(|running| running.get()).unwrap_or(true)
                fallback=|| view! {
                    <div style="
                        display: flex;
                        flex-direction: column;
                        justify-content: center;
                        align-items: center;
                        width: 100%;
                        height: 100%;
                    ">
                        <h4 style="
                            maxWidth: min(100%, 400px);
                            textAlign: center;
                        ">
                            "Click 'Start Engine' to see the output."
                        </h4>
                    </div>
                }
            >
                <EngineCanvas tx=engine_tx />
            </Show>
        </div>
    }
}

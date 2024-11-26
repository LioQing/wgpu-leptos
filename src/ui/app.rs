use std::sync::mpsc;

use leptos::*;
use leptos_use::use_element_size;

use crate::{systems, ui::components::EngineCanvas};

const INSTRUCTIONS: &[&str] = &[
    "Click the button to start or stop the engine.",
    "Click on the canvas to focus and lock the cursor.",
    "Move the mouse to look around when the cursor is locked.",
    "Use the W, A, S, D, Space, Shift keys to move around when the cursor is locked.",
    "Press the Escape key or the Tab key to unlock the cursor.",
];

#[component]
pub fn App() -> impl IntoView {
    let container_node = create_node_ref::<html::Div>();
    let container_size = use_element_size(container_node);

    let engine_tx = create_rw_signal::<Option<mpsc::Sender<systems::EngineExternalSignal>>>(None);
    let running = create_rw_signal(false);

    // Keep the engine same size as the container.
    create_effect(move |_| {
        let _ = running.get();
        let _ = container_size.width.get();
        let _ = container_size.height.get();

        // We can only resize the engine if the container node and engine tx are available.
        let (container_node, tx) = match (container_node.get(), engine_tx.get()) {
            (Some(container_node), Some(tx)) => (container_node, tx),
            _ => return,
        };

        let container = container_node.get_bounding_client_rect();
        let width = container.width();
        let height = container.height();

        log::debug!("Resizing engine to {width} x {height}");
        tx.send(systems::ResizeSignal::queued(width, height))
            .unwrap();
    });

    view! {
        <div style="
            display: flex;
            flex-direction: column;
            height: 100vh;
            overflow: hidden;
        ">
            <div style="flex: 1; display: flex; height: 100%; overflow: hidden;">
                <div style="
                    flex: 1 0 50px;
                    display: flex;
                    flex-direction: column;
                    overflow: auto;
                    padding: 16px 24px;
                ">
                    <h2>"wgpu + Leptos"</h2>
                    <div style="display: flex;">
                        <button on:click=move |_| running.set(!running.get())>
                            <Show
                                when=move || running.get()
                                fallback=|| "Start Engine"
                            >
                                "Stop Engine"
                            </Show>
                        </button>
                    </div>
                    <h3>"Instructions"</h3>
                    <ul style="margin-top: 0;">
                        {INSTRUCTIONS
                            .iter()
                            .map(|instruction| view! { <li>{*instruction}</li> })
                            .collect_view()
                        }
                    </ul>
                </div>
                <div ref=container_node style="flex: 3 0 100px; height=100%;">
                    <Show
                        when=move || running.get()
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
                                    "Click 'Start Engine' to start the engine."
                                </h4>
                            </div>
                        }
                    >
                        <EngineCanvas tx=engine_tx />
                    </Show>
                </div>
            </div>
            <footer style="
                flex: 0 0 auto;
                display: flex;
                justify-content: center;
                align-items: center;
                background-color: lightgray;
            ">
                <p>
                    "Made with ❤️ by "
                    <a href="https://lioqing.com" target="_blank">
                        " Lio Qing"
                    </a>
                    " | "
                    <a href="https://github.com/lioqing/wgpu-leptos-template" target="_blank">
                        "GitHub Repository"
                    </a>
                </p>
            </footer>
        </div>
    }
}

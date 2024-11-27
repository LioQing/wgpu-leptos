use leptos::*;

use crate::ui::components::{engine::EngineController, PyramidTransformConfiguration};

const INSTRUCTIONS: &[&str] = &[
    "Click the button to start or stop the engine.",
    "Click on the canvas to focus and lock the cursor.",
    "Move the mouse to look around when the cursor is locked.",
    "Use the W, A, S, D, Space, Shift keys to move around when the cursor is locked.",
    "Change the configurations to see the changes in real-time.",
    "Press the Escape key or the Tab key to unlock the cursor.",
];

#[component]
pub fn SidePanel(
    #[prop(into)] controller: EngineController,
    #[prop(default = "".to_string(), into)] style: String,
) -> impl IntoView {
    view! {
        <div style="\
            display: flex;\
            flex-direction: column;\
            overflow: auto;\
            padding: 16px 24px;\
        ".to_string() + &style>
            <h2>"wgpu + Leptos"</h2>
            <div style="display: flex;">
                <button on:click=move |_| controller.running().set(!controller.running().get())>
                    <Show
                        when=move || controller.running().get()
                        fallback=|| "Start Engine"
                    >
                        "Stop Engine"
                    </Show>
                </button>
            </div>
            <div style="margin-bottom: 16px;" />
            <h3 style="margin-top: 0;">"Configurations"</h3>
            <PyramidTransformConfiguration controller=controller />
            <div style="margin-bottom: 16px;" />
            <h3 style="margin-top: 0;">"Instructions"</h3>
            <ul style="margin-top: 0;">
                {INSTRUCTIONS
                    .iter()
                    .map(|instruction| view! { <li>{*instruction}</li> })
                    .collect_view()
                }
            </ul>
        </div>
    }
}

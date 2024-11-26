use leptos::*;

const INSTRUCTIONS: &[&str] = &[
    "Click the button to start or stop the engine.",
    "Click on the canvas to focus and lock the cursor.",
    "Move the mouse to look around when the cursor is locked.",
    "Use the W, A, S, D, Space, Shift keys to move around when the cursor is locked.",
    "Press the Escape key or the Tab key to unlock the cursor.",
];

#[component]
pub fn SidePanel(
    #[prop(into)] running: RwSignal<bool>,
    #[prop(default = "".to_string(), into)] style: String,
) -> impl IntoView {
    view! {
        <div style="
            display: flex;
            flex-direction: column;
            overflow: auto;
            padding: 16px 24px;
        ".to_string() + &style>
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
    }
}

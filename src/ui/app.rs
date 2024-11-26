use leptos::*;

use crate::ui::components::{Engine, Footer, SidePanel};

#[component]
pub fn App() -> impl IntoView {
    let running = create_rw_signal(false);

    view! {
        <div style="
            display: flex;
            flex-direction: column;
            height: 100vh;
            overflow: hidden;
        ">
            <div style="flex: 1; display: flex; height: 100%; overflow: hidden;">
                <SidePanel running=running style="flex: 1 0 50px;" />
                <Engine running=running style="flex: 3 0 100px; height=100%;" />
            </div>
            <Footer style="flex: 0 0 auto;" />
        </div>
    }
}

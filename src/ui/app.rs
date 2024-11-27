use leptos::*;

use crate::ui::components::{engine::EngineController, Engine, Footer, SidePanel};

#[component]
pub fn App() -> impl IntoView {
    let controller = EngineController::default();

    view! {
        <div style="\
            display: flex; \
            flex-direction: column; \
            height: 100vh; \
            overflow: hidden; \
        ">
            <div style="flex: 1; display: flex; height: 100%; overflow: hidden;">
                <SidePanel controller=controller style="flex: 0 0 300px;" />
                <Engine controller=controller style="flex: 1 1 auto; height: 100%;" />
            </div>
            <Footer style="flex: 0 0 auto;" />
        </div>
    }
}

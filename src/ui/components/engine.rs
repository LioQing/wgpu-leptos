use leptos::*;
use leptos_use::use_element_bounding;

use crate::{
    systems,
    ui::components::{engine_canvas::EngineTx, EngineCanvas},
};

/// Engine component.
#[component]
pub fn Engine(
    #[prop(optional, into)] controller: Option<EngineController>,
    #[prop(default = "".to_string(), into)] style: String,
) -> impl IntoView {
    let container_node = create_node_ref::<html::Div>();
    let container_size = use_element_bounding(container_node);

    let controller = controller.unwrap_or_default();

    // Keep the engine same size as the container.
    create_effect(move |_| {
        controller.running.get();
        container_size.width.get();
        container_size.height.get();

        // We can only resize the engine if the container node and engine tx are available.
        let (container_node, tx) = match (container_node.get(), controller.tx().get()) {
            (Some(container_node), Some(tx)) => (container_node, tx),
            _ => return,
        };

        let container = container_node.get_bounding_client_rect();
        let width = container.width();
        let height = container.height();

        tx.send(systems::ResizeSignal::new(width, height)).unwrap();
    });

    view! {
        <div ref=container_node style=format!("overflow: hidden; {style}")>
            <Show
                when=move || controller.running().get()
                fallback=|| view! {
                    <div style="\
                        display: flex; \
                        flex-direction: column; \
                        justify-content: center; \
                        align-items: center; \
                        width: 100%; \
                        height: 100%; \
                    ">
                        <h4 style="\
                            maxWidth: min(100%, 400px); \
                            textAlign: center; \
                        ">
                            "Click 'Start Engine' to see the output."
                        </h4>
                    </div>
                }
            >
                <EngineCanvas
                    system_pipeline_args=move || systems::Args {
                        pyramid_transform: controller.pyramid_transform().get(),
                        pyramid_model: controller.pyramid_model().get(),
                        ..Default::default()
                    }
                    tx=controller.tx().split()
                />
            </Show>
        </div>
    }
}

/// Engine controller.
#[derive(Debug, Clone, Copy)]
pub struct EngineController {
    running: RwSignal<bool>,
    tx: RwSignal<EngineTx>,
    pyramid_transform: RwSignal<systems::handlers::PyramidTransform>,
    pyramid_model: RwSignal<systems::handlers::PyramidModel>,
}

impl EngineController {
    pub fn running(&self) -> RwSignal<bool> {
        self.running
    }

    pub fn tx(&self) -> RwSignal<EngineTx> {
        self.tx
    }

    pub fn pyramid_transform(&self) -> RwSignal<systems::handlers::PyramidTransform> {
        self.pyramid_transform
    }

    pub fn signal_pyramid_transform_update(&self) {
        self.tx().with(|tx| match tx {
            Some(tx) => {
                tx.send(systems::PyramidTransformUpdateSignal::new(
                    self.pyramid_transform().get(),
                ))
                .unwrap();
            }
            None => log::debug!("Engine has not started, skipping signal pyramid transform"),
        });
    }

    pub fn pyramid_model(&self) -> RwSignal<systems::handlers::PyramidModel> {
        self.pyramid_model
    }
}

impl Default for EngineController {
    fn default() -> Self {
        let running = create_rw_signal(false);
        let tx = create_rw_signal(None);
        let pyramid_transform = create_rw_signal(systems::handlers::PyramidTransform::default());
        let pyramid_model = create_rw_signal(systems::handlers::PyramidModel::default());

        Self {
            running,
            tx,
            pyramid_transform,
            pyramid_model,
        }
    }
}

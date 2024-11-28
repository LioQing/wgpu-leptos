use leptos::*;
use leptos_use::{use_element_bounding, use_interval_fn};

use crate::{
    systems,
    ui::components::{
        engine_canvas::{EngineRx, EngineTx},
        EngineCanvas,
    },
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

    // Add handler for engine's signal.
    controller.add_rx_handler(move |signal| match signal {
        systems::Signal::PyramidTransformUpdate(signal) => {
            controller.pyramid_transform.set(signal.transform);
        }
        systems::Signal::PyramidModelUpdate(signal) => {
            controller.pyramid_model.set(signal.model);
        }
        _ => log::warn!("Unhandled signal: {signal:?}"),
    });

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

        tx.send(systems::ResizeSignal::in_signal(width, height))
            .unwrap();
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
                    rx=controller.rx().split()
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
    rx: RwSignal<EngineRx>,
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

    pub fn rx(&self) -> RwSignal<EngineRx> {
        self.rx
    }

    pub fn pyramid_transform(&self) -> RwSignal<systems::handlers::PyramidTransform> {
        self.pyramid_transform
    }

    pub fn signal_pyramid_transform_update(&self) {
        self.tx().with(|tx| match tx {
            Some(tx) => {
                tx.send(systems::PyramidTransformUpdateSignal::in_signal(
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

impl EngineController {
    pub fn add_rx_handler(&self, handler: impl Fn(systems::Signal) + Clone + 'static) {
        let rx = self.rx;
        use_interval_fn(
            move || {
                rx.with(|rx: &EngineRx| {
                    if let Some(rx) = rx {
                        for signal in rx.try_iter() {
                            handler(signal);
                        }
                    }
                })
            },
            (systems::Args::default()
                .fps_limit
                .as_secs_f32()
                .map(|v| v.sqrt()) // Make it more responsive
                .unwrap_or(1e-3)
                * 1e3) as u64,
        );
    }
}

impl Default for EngineController {
    fn default() -> Self {
        let running = create_rw_signal(false);
        let tx = create_rw_signal(None);
        let rx = create_rw_signal(None);
        let pyramid_transform = create_rw_signal(systems::handlers::PyramidTransform::default());
        let pyramid_model = create_rw_signal(systems::handlers::PyramidModel::default());

        Self {
            running,
            tx,
            rx,
            pyramid_transform,
            pyramid_model,
        }
    }
}

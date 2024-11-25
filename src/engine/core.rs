use std::{
    collections::VecDeque,
    sync::{mpsc, Arc},
};

use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};
use winit_input_helper::WinitInputHelper;

use crate::engine::{ExternalSignal, Items, SystemPipeline};

/// The main engine struct that create the window and runs the system pipeline.
pub struct Engine<T: SystemPipeline<Args = U, ExternalSignal = V>, U: 'static, V> {
    external_rx: Option<mpsc::Receiver<ExternalSignal<U, V>>>,
    queued_signals: VecDeque<V>,
    state: EngineState<T, U>,
}

impl<T: SystemPipeline<Args = U, ExternalSignal = V>, U, V> Engine<T, U, V> {
    pub fn new(window_attributes: WindowAttributes, system_pipeline_args: U) -> Self {
        let state = EngineState::PreInit {
            items: PreInitItems {
                window_attributes,
                system_pipeline_args,
            },
        };

        Self {
            external_rx: None,
            queued_signals: VecDeque::new(),
            state,
        }
    }

    /// Set the external signal receiver.
    pub fn with_external_signal_rx(mut self, rx: mpsc::Receiver<ExternalSignal<U, V>>) -> Self {
        self.external_rx = Some(rx);
        self
    }
}

impl<T: SystemPipeline<Args = U, ExternalSignal = V>, U, V> ApplicationHandler for Engine<T, U, V> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Already initialized
        if !matches!(&self.state, EngineState::PreInit { .. }) {
            log::warn!("Engine already initializing or initialized");
            return;
        }

        // Take the pre-init items and start initializing the engine
        let PreInitItems {
            window_attributes,
            system_pipeline_args,
        } = self.state.initialize_engine();

        // Set up window and control flow
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;

            window.set_prevent_default(true);
        }

        log::debug!("Window created");

        // Initialize system pipeline
        let (tx, rx) = mpsc::channel();
        let init_fn = async move {
            let system_pipeline = T::init(window.clone(), system_pipeline_args).await;
            tx.send((window.clone(), system_pipeline)).unwrap();
            window.request_redraw();
        };

        log::debug!("Spawning system pipeline initialization future");

        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                wasm_bindgen_futures::spawn_local(init_fn);
            } else {
                futures::executor::block_on(init_fn);
            }
        }

        // Update state
        self.state = EngineState::InitializingSystemPipeline {
            init_rx: rx,
            input: Box::new(WinitInputHelper::new()),
        };

        log::info!("System pipeline initializing asynchronously");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        // Shut down if the window is closed
        if let WindowEvent::CloseRequested = event {
            log::info!("Engine exiting");
            event_loop.exit();
        }

        match &mut self.state {
            EngineState::InitializingSystemPipeline { init_rx, input } => {
                input.window_event(&event);

                // Wait for the system pipeline to initialize
                if let WindowEvent::RedrawRequested = event {
                    if let Ok((window, system_pipeline)) = init_rx.try_recv() {
                        window.request_redraw();

                        self.state = EngineState::PostInit {
                            items: Items {
                                window,
                                input: std::mem::take(input),
                            },
                            system_pipeline,
                        };
                        log::info!("Engine initialized")
                    }
                }
            }
            EngineState::PostInit {
                items,
                system_pipeline,
            } => {
                // Call system pipeline `window_event`
                system_pipeline.window_event(items, &event);

                items.input.window_event(&event);

                if let WindowEvent::RedrawRequested = event {
                    // Call system pipeline `update`
                    system_pipeline.update(items);

                    items.input.end_step();
                    items.input.new_events();
                }

                // Handle queued events
                while let Some(signal) = self.queued_signals.pop_front() {
                    system_pipeline.external_signal(items, signal);
                }

                // Handle external events
                if let Some(rx) = &self.external_rx {
                    for signal in rx.try_iter() {
                        match signal {
                            ExternalSignal::Stop => {
                                log::info!("Engine stopping");
                                self.state = EngineState::Stopped {
                                    window: items.window.clone(),
                                };
                                return;
                            }
                            ExternalSignal::Start { .. } => log::warn!("Engine already started"),
                            ExternalSignal::Custom { signal, .. } => {
                                system_pipeline.external_signal(items, signal)
                            }
                        }
                    }
                }
            }
            EngineState::Stopped { window } => {
                // Handle external events
                if let Some(rx) = &self.external_rx {
                    for signal in rx.try_iter() {
                        match signal {
                            ExternalSignal::Start {
                                window_attributes,
                                system_pipeline_args,
                            } => {
                                log::info!("Engine restarting");
                                self.state = EngineState::PreInit {
                                    items: PreInitItems {
                                        window_attributes,
                                        system_pipeline_args,
                                    },
                                };
                                self.resumed(event_loop);
                                return;
                            }
                            ExternalSignal::Stop => log::warn!("Engine already stopped"),
                            ExternalSignal::Custom { signal, queued } => match queued {
                                true => self.queued_signals.push_front(signal),
                                false => log::warn!("Custom signal received but engine is stopped, you may try to use queued signals"),
                            }
                        }
                    }

                    window.request_redraw();
                } else {
                    panic!("Engine stopped without external receiver");
                }
            }
            state => log::error!("Engine in unexpected state: {state:?}"),
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        match &mut self.state {
            EngineState::InitializingSystemPipeline { input, .. } => {
                input.device_event(&event);
            }
            EngineState::PostInit {
                items,
                system_pipeline,
                ..
            } => {
                items.input.device_event(&event);

                // Call system pipeline `device_event`
                system_pipeline.device_event(items, &event);
            }
            _ => {}
        }
    }
}

struct PreInitItems<T> {
    window_attributes: WindowAttributes,
    system_pipeline_args: T,
}

/// Items representing the engine's initialization state.
enum EngineState<T, U> {
    PreInit {
        items: PreInitItems<U>,
    },
    InitializingEngine,
    InitializingSystemPipeline {
        init_rx: mpsc::Receiver<(Arc<Window>, T)>,
        input: Box<WinitInputHelper>,
    },
    PostInit {
        items: Items,
        system_pipeline: T,
    },
    Stopped {
        window: Arc<Window>,
    },
}

impl<T, U> EngineState<T, U> {
    /// Take the [`PreInitItems<U>`] and go into
    /// [`EngineState::InitializingEngine`].
    fn initialize_engine(&mut self) -> PreInitItems<U> {
        match std::mem::replace(self, Self::InitializingEngine) {
            Self::PreInit { items } => items,
            state => panic!("Expected `PreInit`, found {state:?}"),
        }
    }
}

impl<T, U> std::fmt::Debug for EngineState<T, U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PreInit { .. } => write!(f, "EngineState::PreInit"),
            Self::InitializingEngine => write!(f, "EngineState::InitializingEngine"),
            Self::InitializingSystemPipeline { .. } => {
                write!(f, "EngineState::InitializingSystemPipeline")
            }
            Self::PostInit { .. } => write!(f, "EngineState::PostInit"),
            Self::Stopped { .. } => write!(f, "EngineState::Stopped"),
        }
    }
}

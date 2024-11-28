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

use crate::engine::{signal, InSignal, Items, SystemPipeline};

/// The main engine struct that create the window and runs the system pipeline.
pub struct Engine<T: SystemPipeline> {
    rx: Option<mpsc::Receiver<InSignal<T>>>,
    tx: Option<mpsc::Sender<T::OutSignal>>,
    queued_signals: VecDeque<T::InSignal>,
    state: EngineState<T>,
}

impl<T: SystemPipeline> Engine<T> {
    pub fn new(window_attributes: WindowAttributes, system_pipeline_args: T::Args) -> Self {
        let state = EngineState::PreInit {
            items: PreInitItems {
                window_attributes,
                system_pipeline_args,
            },
        };

        Self {
            rx: None,
            tx: None,
            queued_signals: VecDeque::new(),
            state,
        }
    }

    /// Set the incoming signal receiver.
    pub fn with_rx(mut self, rx: mpsc::Receiver<InSignal<T>>) -> Self {
        self.rx = Some(rx);
        self
    }

    /// Set the outgoing signal sender.
    pub fn with_tx(mut self, tx: mpsc::Sender<T::OutSignal>) -> Self {
        self.tx = Some(tx);
        self
    }
}

impl<T: SystemPipeline> ApplicationHandler for Engine<T> {
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
                            items: Items::<T::OutSignal> {
                                window,
                                input: std::mem::take(input),
                                tx: self.tx.clone(),
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
                    system_pipeline.in_signal(items, signal);
                }

                // Handle incoming events
                if let Some(rx) = &self.rx {
                    for signal in rx.try_iter() {
                        match signal {
                            InSignal::Stop => {
                                log::info!("Engine stopping");
                                self.state = EngineState::Stopped {
                                    window: items.window.clone(),
                                };
                                return;
                            }
                            InSignal::Start { .. } => log::warn!("Engine already started"),
                            InSignal::Custom { signal, .. } => {
                                system_pipeline.in_signal(items, signal)
                            }
                        }
                    }
                }
            }
            EngineState::Stopped { window } => {
                // Handle incoming events
                if let Some(rx) = &self.rx {
                    for signal in rx.try_iter() {
                        match signal {
                            InSignal::Start {
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
                            InSignal::Stop => log::warn!("Engine already stopped"),
                            InSignal::Custom { signal, queue } => match queue {
                                signal::QueueBehavior::Replace(pred) => {
                                    self.queued_signals.retain(|x| !pred(x, &signal));
                                    self.queued_signals.push_back(signal)
                                }
                                signal::QueueBehavior::Queued => {
                                    self.queued_signals.push_back(signal)
                                }
                                signal::QueueBehavior::Ignored => {}
                            },
                        }
                    }

                    window.request_redraw();
                } else {
                    panic!("Engine stopped without incoming signal receiver");
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
enum EngineState<T: SystemPipeline> {
    PreInit {
        items: PreInitItems<T::Args>,
    },
    InitializingEngine,
    InitializingSystemPipeline {
        init_rx: mpsc::Receiver<(Arc<Window>, T)>,
        input: Box<WinitInputHelper>,
    },
    PostInit {
        items: Items<T::OutSignal>,
        system_pipeline: T,
    },
    Stopped {
        window: Arc<Window>,
    },
}

impl<T: SystemPipeline> EngineState<T> {
    /// Take the [`PreInitItems<T::Args>`] and go into
    /// [`EngineState::InitializingEngine`].
    fn initialize_engine(&mut self) -> PreInitItems<T::Args> {
        match std::mem::replace(self, Self::InitializingEngine) {
            Self::PreInit { items } => items,
            state => panic!("Expected `PreInit`, found {state:?}"),
        }
    }
}

impl<T: SystemPipeline> std::fmt::Debug for EngineState<T> {
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

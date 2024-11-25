use std::sync::mpsc;

use winit::{
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

use crate::engine::{Engine, Error, ExternalSignal, SystemPipeline};

/// Build and run engine.
pub struct Runner<T, U> {
    window_attributes: WindowAttributes,
    system_pipeline: T,
    external_signal_rx: U,
}

pub struct NoSystemPipeline;
pub struct WithSystemPipeline<T: SystemPipeline>(pub T::Args);

pub struct NoExternalSignalRx;
pub struct WithExternalSignalRx<T, U>(pub mpsc::Receiver<ExternalSignal<T, U>>);

impl Runner<NoSystemPipeline, NoExternalSignalRx> {
    /// Create a new runner.
    pub fn new() -> Self {
        Self {
            window_attributes: Window::default_attributes().with_title("wgpu + Leptos"),
            system_pipeline: NoSystemPipeline,
            external_signal_rx: NoExternalSignalRx,
        }
    }
}

impl<T, U> Runner<T, U> {
    /// Set the window attributes.
    pub fn with_window_attributes(self, window_attributes: WindowAttributes) -> Self {
        Self {
            window_attributes,
            ..self
        }
    }

    /// Set the system pipeline to use, and the arguments to pass to
    /// [`SystemPipeline::init`].
    pub fn with_system_pipeline<W: SystemPipeline>(
        self,
        args: W::Args,
    ) -> Runner<WithSystemPipeline<W>, U> {
        Runner {
            window_attributes: self.window_attributes,
            system_pipeline: WithSystemPipeline(args),
            external_signal_rx: self.external_signal_rx,
        }
    }

    /// Set receiver to listen for any events from outside the engine.
    pub fn with_external_signal_rx<W, X>(
        self,
        rx: mpsc::Receiver<ExternalSignal<W, X>>,
    ) -> Runner<T, WithExternalSignalRx<W, X>> {
        Runner {
            window_attributes: self.window_attributes,
            system_pipeline: self.system_pipeline,
            external_signal_rx: WithExternalSignalRx(rx),
        }
    }
}

impl<T: SystemPipeline<Args = U, ExternalSignal = V>, U: 'static, V>
    Runner<WithSystemPipeline<T>, WithExternalSignalRx<U, V>>
{
    /// Run the engine.
    pub fn run(self) -> Result<(), Error> {
        let event_loop = EventLoop::new()?;
        let mut engine = Engine::<T, U, V>::new(self.window_attributes, self.system_pipeline.0)
            .with_external_signal_rx(self.external_signal_rx.0);

        log::info!("Starting engine");
        Ok(event_loop.run_app(&mut engine)?)
    }
}

impl<T: SystemPipeline<Args = U>, U: 'static> Runner<WithSystemPipeline<T>, NoExternalSignalRx> {
    /// Run the engine.
    pub fn run(self) -> Result<(), Error> {
        let event_loop = EventLoop::new()?;
        let mut engine = Engine::<T, U, _>::new(self.window_attributes, self.system_pipeline.0);

        log::info!("Starting engine");
        Ok(event_loop.run_app(&mut engine)?)
    }
}

use std::sync::mpsc;

use winit::{
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

use crate::engine::{Engine, Error, InSignal, SystemPipeline};

/// Build and run engine.
pub struct Runner<T, U> {
    window_attributes: WindowAttributes,
    system_pipeline: T,
    rx: U,
}

pub struct NoSystemPipeline;
pub struct WithSystemPipeline<T: SystemPipeline>(pub T::Args);

pub struct NoRx;
pub struct WithRx<T, U>(pub mpsc::Receiver<InSignal<T, U>>);

impl Runner<NoSystemPipeline, NoRx> {
    /// Create a new runner.
    pub fn new() -> Self {
        Self {
            window_attributes: Window::default_attributes().with_title("wgpu + Leptos"),
            system_pipeline: NoSystemPipeline,
            rx: NoRx,
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
            rx: self.rx,
        }
    }

    /// Set receiver to listen for any events from outside the engine.
    pub fn with_rx<W, X>(self, rx: mpsc::Receiver<InSignal<W, X>>) -> Runner<T, WithRx<W, X>> {
        Runner {
            window_attributes: self.window_attributes,
            system_pipeline: self.system_pipeline,
            rx: WithRx(rx),
        }
    }
}

impl<T: SystemPipeline<Args = U, InSignal = V>, U: 'static, V>
    Runner<WithSystemPipeline<T>, WithRx<U, V>>
{
    /// Run the engine.
    pub fn run(self) -> Result<(), Error> {
        let event_loop = EventLoop::new()?;
        let mut engine = Engine::<T, U, V>::new(self.window_attributes, self.system_pipeline.0)
            .with_rx(self.rx.0);

        log::info!("Starting engine");
        Ok(event_loop.run_app(&mut engine)?)
    }
}

impl<T: SystemPipeline<Args = U>, U: 'static> Runner<WithSystemPipeline<T>, NoRx> {
    /// Run the engine.
    pub fn run(self) -> Result<(), Error> {
        let event_loop = EventLoop::new()?;
        let mut engine = Engine::<T, U, _>::new(self.window_attributes, self.system_pipeline.0);

        log::info!("Starting engine");
        Ok(event_loop.run_app(&mut engine)?)
    }
}

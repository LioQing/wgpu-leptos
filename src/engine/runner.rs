use std::sync::mpsc;

use winit::{
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

use crate::engine::{Engine, Error, InSignal, SystemPipeline};

/// Build and run engine.
pub struct Runner<T, U, V> {
    window_attributes: WindowAttributes,
    system_pipeline: T,
    rx: U,
    tx: V,
}

pub struct NoSystemPipeline;
pub struct WithSystemPipeline<T: SystemPipeline>(pub T::Args);

pub struct NoRx;
pub struct WithRx<T: SystemPipeline>(pub mpsc::Receiver<InSignal<T>>);

pub struct NoTx;
pub struct WithTx<T: SystemPipeline>(pub mpsc::Sender<T::OutSignal>);

impl Runner<NoSystemPipeline, NoRx, NoTx> {
    /// Create a new runner.
    pub fn new() -> Self {
        Self {
            window_attributes: Window::default_attributes().with_title("wgpu + Leptos"),
            system_pipeline: NoSystemPipeline,
            rx: NoRx,
            tx: NoTx,
        }
    }
}

impl<T, U, V> Runner<T, U, V> {
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
    ) -> Runner<WithSystemPipeline<W>, U, V> {
        Runner {
            window_attributes: self.window_attributes,
            system_pipeline: WithSystemPipeline(args),
            rx: self.rx,
            tx: self.tx,
        }
    }

    /// Set receiver to listen for any events from outside the engine.
    pub fn with_rx<W: SystemPipeline>(
        self,
        rx: mpsc::Receiver<InSignal<W>>,
    ) -> Runner<T, WithRx<W>, V> {
        Runner {
            window_attributes: self.window_attributes,
            system_pipeline: self.system_pipeline,
            rx: WithRx(rx),
            tx: self.tx,
        }
    }

    /// Set sender to send any events to outside the engine.
    pub fn with_tx<W: SystemPipeline>(
        self,
        tx: mpsc::Sender<W::OutSignal>,
    ) -> Runner<T, U, WithTx<W>> {
        Runner {
            window_attributes: self.window_attributes,
            system_pipeline: self.system_pipeline,
            rx: self.rx,
            tx: WithTx(tx),
        }
    }
}

impl<T: SystemPipeline> Runner<WithSystemPipeline<T>, WithRx<T>, WithTx<T>> {
    /// Run the engine.
    pub fn run(self) -> Result<(), Error> {
        let event_loop = EventLoop::new()?;
        let mut engine = Engine::<T>::new(self.window_attributes, self.system_pipeline.0)
            .with_rx(self.rx.0)
            .with_tx(self.tx.0);

        log::info!("Starting engine");
        Ok(event_loop.run_app(&mut engine)?)
    }
}

impl<T: SystemPipeline> Runner<WithSystemPipeline<T>, NoRx, NoTx> {
    /// Run the engine.
    pub fn run(self) -> Result<(), Error> {
        let event_loop = EventLoop::new()?;
        let mut engine = Engine::<T>::new(self.window_attributes, self.system_pipeline.0);

        log::info!("Starting engine");
        Ok(event_loop.run_app(&mut engine)?)
    }
}

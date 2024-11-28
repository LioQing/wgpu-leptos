use std::sync::Arc;

use winit::{
    event::{DeviceEvent, WindowEvent},
    window::Window,
};

use crate::engine::Items;

#[allow(unused_variables)]
/// Trait for the system pipeline that the engine will run.
pub trait SystemPipeline: Sized + 'static {
    /// Arguments passed in externally to the engine for initialization.
    type Args;

    /// Custom incoming signal for [`crate::engine::InSignal::Custom`].
    type InSignal;

    /// Called when the window is just created.
    async fn init(window: Arc<Window>, args: Self::Args) -> Self;

    /// Called when there is a [`winit::event::DeviceEvent`].
    ///
    /// This is called independently of all other events.
    fn device_event(&mut self, items: &mut Items, event: &DeviceEvent) {}

    /// Called when there is a [`winit::event::WindowEvent`].
    ///
    /// This is called before [`Items::input`] is processed and
    /// [`SystemPipeline::update`].
    fn window_event(&mut self, items: &mut Items, event: &WindowEvent) {}

    /// Called every frame.
    ///
    /// This is called after [`SystemPipeline::window_event`] and
    /// [`Items::input`] is processed.
    fn update(&mut self, items: &mut Items) {}

    /// Called when there is a [`SystemPipeline::InSignal`].
    ///
    /// This is called after [`SystemPipeline::window_event`] and
    /// [`SystemPipeline::update`].
    fn in_signal(&mut self, items: &mut Items, signal: Self::InSignal) {}
}

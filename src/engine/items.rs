use std::sync::{mpsc, Arc};

use winit::window::Window;
use winit_input_helper::WinitInputHelper;

/// Items in the engine.
pub struct Items<T> {
    /// The window.
    pub window: Arc<Window>,

    /// Input helper.
    pub input: WinitInputHelper,

    /// Outgoing signal sender.
    pub tx: Option<mpsc::Sender<T>>,
}

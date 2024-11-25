use std::sync::Arc;

use winit::window::Window;
use winit_input_helper::WinitInputHelper;

/// Items in the engine.
pub struct Items {
    /// The window.
    pub window: Arc<Window>,

    /// Input helper.
    pub input: WinitInputHelper,
}

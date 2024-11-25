use winit::window::WindowAttributes;

/// External signal passed to the engine.
///
/// This is mostly only used in WASM builds,
/// so that the engine can interoperate with the UI.
#[derive(Debug, Clone)]
pub enum ExternalSignal<T, U> {
    /// Start or restart the engine.
    Start {
        window_attributes: WindowAttributes,
        system_pipeline_args: T,
    },
    /// Stop the engine.
    Stop,
    /// Custom signal.
    Custom { signal: U, queued: bool },
}

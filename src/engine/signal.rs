use winit::window::WindowAttributes;

use crate::engine::SystemPipeline;

/// Ingcoming signal passed to the engine.
///
/// This is mostly only used in WASM builds,
/// so that the engine can interoperate with the UI.
#[derive(Debug, Clone)]
pub enum InSignal<T: SystemPipeline> {
    /// Start or restart the engine.
    Start {
        window_attributes: WindowAttributes,
        system_pipeline_args: T::Args,
    },
    /// Stop the engine.
    Stop,
    /// Custom signal.
    Custom {
        signal: T::InSignal,
        queue: QueueBehavior<T::InSignal>,
    },
}

/// Queue behavior of the [`InSignal`].
///
/// The engine provides a queue for signals when it is stopped,
/// this enum specifies the behavior of the signal when queued.
#[derive(Debug, Clone, Copy)]
pub enum QueueBehavior<U> {
    /// The signal is ignored.
    Ignored,
    /// All the matching signals are replaced.
    ///
    /// A comparison function is provided to determine if two signals are matching.
    /// The first argument is the new signal, and the second argument is the old signal.
    Replace(fn(&U, &U) -> bool),
    /// The signal is queued.
    Queued,
}

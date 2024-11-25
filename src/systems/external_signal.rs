#![allow(clippy::new_ret_no_self)]

use crate::{engine, systems::Args};

/// External signal of [`Pipeline`].
pub enum ExternalSignal {
    Resize(ResizeSignal),
}

/// Type alias for the [`engine::ExternalSignal`] of [`crate::systems::Pipeline`].
pub type EngineExternalSignal = engine::ExternalSignal<Args, ExternalSignal>;

/// Resize signal.
pub struct ResizeSignal {
    pub width: f64,
    pub height: f64,
}

impl ResizeSignal {
    pub fn new(width: f64, height: f64) -> EngineExternalSignal {
        engine::ExternalSignal::Custom {
            signal: ExternalSignal::Resize(Self { width, height }),
            queued: false,
        }
    }

    pub fn queued(width: f64, height: f64) -> EngineExternalSignal {
        engine::ExternalSignal::Custom {
            signal: ExternalSignal::Resize(Self { width, height }),
            queued: true,
        }
    }
}

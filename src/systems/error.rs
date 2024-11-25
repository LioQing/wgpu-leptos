use thiserror::Error;

use crate::systems::ColorError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("display lock cursor error: {0}")]
    DisplayLockCursor(#[from] winit::error::ExternalError),

    #[error("color error: {0}")]
    Color(#[from] ColorError),
}

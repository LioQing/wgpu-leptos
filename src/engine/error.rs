use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Winit event loop error: {0}")]
    WinitEventLoopError(#[from] winit::error::EventLoopError),
}

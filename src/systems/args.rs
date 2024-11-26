use glam::*;

use crate::systems::{
    handlers::{PyramidModel, PyramidTransform},
    RgbColor,
};

/// The configurations of the system pipeline.
#[derive(Debug, Clone)]
pub struct Args {
    pub fps_limit: FpsLimit,
    pub clear_color: RgbColor,
    pub pyramid_transform: PyramidTransform,
    pub pyramid_model: PyramidModel,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            fps_limit: FpsLimit::default(),
            clear_color: RgbColor::GRAY,
            pyramid_transform: PyramidTransform::default(),
            pyramid_model: PyramidModel::default(),
        }
    }
}

/// The maximum number of frames per second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FpsLimit(u32);

impl FpsLimit {
    const PADDING: f32 = 1e-3;

    pub fn new(fps: u32) -> Self {
        match fps {
            0 | 1000.. => Self::unlimited(),
            _ => Self::limited(fps).expect("fps != 0"),
        }
    }

    pub fn unlimited() -> Self {
        Self(0)
    }

    pub fn limited(fps: u32) -> Option<Self> {
        match fps {
            0 => None,
            _ => Some(Self(fps)),
        }
    }

    pub fn as_secs_f32(&self) -> Option<f32> {
        match self.0 {
            0 => None,
            _ => Some(1.0 / self.0 as f32 - Self::PADDING),
        }
    }
}

impl Default for FpsLimit {
    fn default() -> Self {
        Self::new(60)
    }
}

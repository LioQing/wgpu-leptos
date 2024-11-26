#![allow(clippy::new_ret_no_self)]

use crate::{
    engine,
    systems::{handlers::PyramidModel, Args},
};

use super::handlers::PyramidTransform;

macro_rules! signals {
    ($($name:ident { $($field:ident: $type:ty),* $(,)? }),* $(,)?) => {
        paste::paste! {
            /// External signal of [`Pipeline`].
            pub enum ExternalSignal {
                $($name([<$name Signal>]),)*
            }

            /// Type alias for the [`engine::ExternalSignal`] of [`crate::systems::Pipeline`].
            pub type EngineExternalSignal = engine::ExternalSignal<Args, ExternalSignal>;

            $(
                /// [`[<$name Signal>]`] signal for [`ExternalSignal`].
                pub struct [<$name Signal>] {
                    $(pub $field: $type,)*
                }

                impl [<$name Signal>] {
                    pub fn new($($field: $type),*) -> EngineExternalSignal {
                        Self { $($field),* }.into_external_signal(false)
                    }

                    pub fn queued($($field: $type),*) -> EngineExternalSignal {
                        Self { $($field),* }.into_external_signal(true)
                    }

                    pub fn into_external_signal(self, queued: bool) -> EngineExternalSignal {
                        engine::ExternalSignal::Custom {
                            signal: ExternalSignal::$name(self),
                            queued,
                        }
                    }
                }
            )*
        }
    };
}

signals! {
    Resize {
        width: f64,
        height: f64,
    },
    PyramidTransformUpdate {
        transform: PyramidTransform,
    },
    PyramidModelUpdate {
        model: PyramidModel,
    },
}

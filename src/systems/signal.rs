#![allow(clippy::new_ret_no_self)]

use crate::{
    engine::{self, signal::QueueBehavior},
    systems::{handlers::PyramidModel, Args},
};

use super::handlers::PyramidTransform;

/// Type alias for the [`engine::InSignal`] of [`crate::systems::Pipeline`].
pub type EngineInSignal = engine::InSignal<Args, InSignal>;

macro_rules! signals {
    (
        $(
            $(#[$($behavior_ident:ident = $behavior_expr:expr),+ $(,)?])?
            $name:ident {
                $($field:ident: $type:ty),* $(,)?
            }
        )*
    ) => {
        paste::paste! {
            /// Incoming signal of [`Pipeline`].
            #[derive(strum::EnumIs)]
            pub enum InSignal {
                $($name([< $name Signal >]),)*
            }

            $(
                /// Signal for [`InSignal`].
                pub struct [< $name Signal >] {
                    $(pub $field: $type,)*
                }

                impl [< $name Signal >] {
                    pub fn new($($field: $type),*) -> EngineInSignal {
                        EngineInSignal::Custom {
                            signal: InSignal::$name(Self { $($field),* }),
                            $($($behavior_ident: $behavior_expr,)+)?
                        }
                    }
                }
            )*
        }
    };
}

signals! {
    #[queue = QueueBehavior::Replace(|a, _| a.is_resize())]
    Resize {
        width: f64,
        height: f64,
    }

    #[queue = QueueBehavior::Ignored]
    PyramidTransformUpdate {
        transform: PyramidTransform,
    }

    #[queue = QueueBehavior::Ignored]
    PyramidModelUpdate {
        model: PyramidModel,
    }
}

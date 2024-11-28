use crate::{
    engine::{self, signal::QueueBehavior},
    systems::{handlers::PyramidModel, Pipeline},
};

use super::handlers::PyramidTransform;

/// Type alias for the [`engine::InSignal`] of [`crate::systems::Pipeline`].
pub type EngineInSignal = engine::InSignal<Pipeline>;

/// Type alias for the [`engine::SystemPipeline::OutSignal`] of [`crate::systems::Pipeline`].
pub type EngineOutSignal = <Pipeline as engine::SystemPipeline>::OutSignal;

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
            /// Incoming and outgoing signal of [`Pipeline`].
            ///
            /// The same type is used for both incoming and outgoing just for simplicity.
            #[derive(strum::EnumIs)]
            pub enum Signal {
                $($name([< $name Signal >]),)*
            }

            impl std::fmt::Debug for Signal {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        $(Signal::$name(..) => write!(f, concat!("Signal::", stringify!($name))),)*
                    }
                }
            }

            $(
                /// Signal for [`Signal`].
                pub struct [< $name Signal >] {
                    $(pub $field: $type,)*
                }

                impl [< $name Signal >] {
                    pub fn in_signal($($field: $type),*) -> EngineInSignal {
                        EngineInSignal::Custom {
                            signal: Signal::$name(Self { $($field),* }),
                            $($($behavior_ident: $behavior_expr,)+)?
                        }
                    }

                    pub fn out_signal($($field: $type),*) -> EngineOutSignal {
                        Signal::$name(Self { $($field),* })
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

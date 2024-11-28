use glam::*;
use leptos::*;
use strum::IntoEnumIterator;

use crate::{systems, ui::components::engine::EngineController};

#[component]
pub fn PyramidTransformConfiguration(#[prop(into)] controller: EngineController) -> impl IntoView {
    let controller = ConfigControllers::new(controller);

    view! {
        <div>
            <h4 style="margin-top: 0; margin-bottom: 16px;">"Pyramid Transform"</h4>
            <div style="display: flex; flex-direction: column; gap: 16px;">
                <Vec3Configuration
                    controller=controller.position()
                    title="Position"
                    min=-10.0
                    max=10.0
                    step=0.05
                />
                <Vec3Configuration
                    controller=controller.scale()
                    title="Scale"
                    min=0.0
                    max=10.0
                    step=0.05
                />
                <ScalarConfiguration
                    controller=controller.auto_rotation_speed()
                    title="Auto Rotation Speed"
                    min=-10.0
                    max=10.0
                    step=0.01
                />
            </div>
        </div>
    }
}

/// Wrapper around [`EngineController`] to handle pyramid transform configurations.
#[derive(Debug, Clone, Copy)]
struct ConfigControllers {
    engine: EngineController,
    position: Vec3Controller,
    scale: Vec3Controller,
    auto_rotation_speed: ScalarController,
}

impl ConfigControllers {
    pub fn new(controller: EngineController) -> Self {
        Self {
            engine: controller,
            position: Vec3Controller::new(
                controller,
                |transform| &transform.transform.position,
                |transform| &mut transform.transform.position,
            ),
            scale: Vec3Controller::new(
                controller,
                |transform| &transform.transform.scale,
                |transform| &mut transform.transform.scale,
            ),
            auto_rotation_speed: ScalarController::new(
                controller,
                |transform| &transform.auto_rotation_speed,
                |transform| &mut transform.auto_rotation_speed,
            ),
        }
    }

    pub fn engine(&self) -> EngineController {
        self.engine
    }

    pub fn position(&self) -> Vec3Controller {
        self.position
    }

    pub fn scale(&self) -> Vec3Controller {
        self.scale
    }

    pub fn auto_rotation_speed(&self) -> ScalarController {
        self.auto_rotation_speed
    }
}

#[derive(Debug, Clone, Copy)]
struct Vec3Controller {
    controller: EngineController,
    property: fn(&systems::handlers::PyramidTransform) -> &Vec3,
    property_mut: fn(&mut systems::handlers::PyramidTransform) -> &mut Vec3,
    err: RwSignal<Option<String>>,
    x: RwSignal<String>,
    y: RwSignal<String>,
    z: RwSignal<String>,
}

impl Vec3Controller {
    pub fn new(
        controller: EngineController,
        property: fn(&systems::handlers::PyramidTransform) -> &Vec3,
        property_mut: fn(&mut systems::handlers::PyramidTransform) -> &mut Vec3,
    ) -> Self {
        let controller_value = controller
            .pyramid_transform()
            .with_untracked(|transform| *property(transform));

        let err = create_rw_signal(None);
        let x = create_rw_signal(controller_value.x.to_string());
        let y = create_rw_signal(controller_value.y.to_string());
        let z = create_rw_signal(controller_value.z.to_string());

        Self {
            controller,
            property,
            property_mut,
            err,
            x,
            y,
            z,
        }
    }

    pub fn reset(&self) {
        let default = *(self.property)(&systems::handlers::PyramidTransform::default());

        self.controller.pyramid_transform().update(|transform| {
            *(self.property_mut)(transform) = default;
        });

        self.controller.signal_pyramid_transform_update();

        self.err().set(None);
        Axis::iter()
            .zip([default.x, default.y, default.z].iter())
            .for_each(|(axis, value)| {
                self.axis(axis).set(value.to_string());
            });
    }

    pub fn err(&self) -> RwSignal<Option<String>> {
        self.err
    }

    pub fn axis(&self, axis: Axis) -> RwSignal<String> {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }

    pub fn sync_engine_axis(&self, axis: Axis) {
        let value = self
            .controller
            .pyramid_transform()
            .with(|transform| match axis {
                Axis::X => (self.property)(transform).x,
                Axis::Y => (self.property)(transform).y,
                Axis::Z => (self.property)(transform).z,
            });

        self.axis(axis).set(value.to_string());
    }

    pub fn set_engine_axis(&self, axis: Axis, value: f32) {
        self.controller.pyramid_transform().update(|transform| {
            let pos = *(self.property)(transform);
            *(self.property_mut)(transform) = match axis {
                Axis::X => vec3(value, pos.y, pos.z),
                Axis::Y => vec3(pos.x, value, pos.z),
                Axis::Z => vec3(pos.x, pos.y, value),
            };
        });

        self.controller.signal_pyramid_transform_update();
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, strum::Display, strum::IntoStaticStr, strum::EnumIter,
)]
enum Axis {
    X,
    Y,
    Z,
}

#[component]
fn Vec3Configuration(
    controller: Vec3Controller,
    #[prop(into)] title: String,
    #[prop(optional)] min: Option<f32>,
    #[prop(optional)] max: Option<f32>,
    #[prop(optional)] step: Option<f32>,
) -> impl IntoView {
    view! {
        <div style="display: flex; flex-direction: column; gap: 8px;">
            <div style="display: flex; gap: 16px; justify-content: space-between;">
                <label for=title.clone()>{title.clone()}</label>
                <button on:click=move |_| controller.reset()>
                    "Reset"
                </button>
            </div>
            <div id=title.clone() style="display: flex; gap: 16px; justify-content: space-between;">
                {Axis::iter()
                    .map(|axis| view! {
                        <Vec3ComponentConfiguration
                            controller=controller
                            axis=axis
                            min=min
                            max=max
                            step=step
                        />
                    })
                    .collect_view()
                }
            </div>
            <Show when=move || controller.err().get().is_some()>
                <div style="color: red;">
                    {format!(
                        "{title} error: {err}",
                        err = controller.err().get().unwrap(),
                    )}
                </div>
            </Show>
        </div>
    }
}

#[component]
fn Vec3ComponentConfiguration(
    controller: Vec3Controller,
    axis: Axis,
    #[prop(optional_no_strip)] min: Option<f32>,
    #[prop(optional_no_strip)] max: Option<f32>,
    #[prop(optional_no_strip)] step: Option<f32>,
) -> impl IntoView {
    let axis_str: &'static str = axis.into();

    view! {
        <div style="display: flex; gap: 8px;">
            <label for=axis_str>{axis_str}</label>
            <input
                id=axis_str
                style="width: 5em;"
                type="number"
                min=min
                max=max
                step=step
                prop:value=controller.axis(axis)
                on:change=move |_| controller.sync_engine_axis(axis)
                on:input=move |event| {
                    // Set the new value
                    let new_value = event_target_value(&event);
                    controller.axis(axis).set(new_value.clone());

                    // Parse the input value.
                    let new_value = match new_value.parse::<f32>() {
                        Ok(new_value) => {
                            controller.err().set(None);
                            new_value.clamp(
                                min.unwrap_or(f32::NEG_INFINITY),
                                max.unwrap_or(f32::INFINITY),
                            )
                        }
                        Err(e) => {
                            controller.err().set(Some(e.to_string()));
                            return;
                        }
                    };

                    // Update the engine value.
                    controller.set_engine_axis(axis, new_value);
                }
            />
        </div>
    }
}

#[derive(Debug, Clone, Copy)]
struct ScalarController {
    controller: EngineController,
    property: fn(&systems::handlers::PyramidTransform) -> &f32,
    property_mut: fn(&mut systems::handlers::PyramidTransform) -> &mut f32,
    err: RwSignal<Option<String>>,
    value: RwSignal<String>,
}

impl ScalarController {
    pub fn new(
        controller: EngineController,
        property: fn(&systems::handlers::PyramidTransform) -> &f32,
        property_mut: fn(&mut systems::handlers::PyramidTransform) -> &mut f32,
    ) -> Self {
        let controller_value = controller
            .pyramid_transform()
            .with_untracked(|transform| *property(transform));

        let err = create_rw_signal(None);
        let value = create_rw_signal(controller_value.to_string());

        Self {
            controller,
            property,
            property_mut,
            err,
            value,
        }
    }

    pub fn reset(&self) {
        let default = *(self.property)(&systems::handlers::PyramidTransform::default());

        self.controller.pyramid_transform().update(|transform| {
            *(self.property_mut)(transform) = default;
        });

        self.controller.signal_pyramid_transform_update();

        self.err().set(None);
        self.value().set(default.to_string());
    }

    pub fn err(&self) -> RwSignal<Option<String>> {
        self.err
    }

    pub fn value(&self) -> RwSignal<String> {
        self.value
    }

    pub fn sync_engine_value(&self) {
        let value = self
            .controller
            .pyramid_transform()
            .with(|transform| *(self.property)(transform));

        self.value().set(value.to_string());
    }

    pub fn set_engine_value(&self, value: f32) {
        self.controller.pyramid_transform().update(|transform| {
            *(self.property_mut)(transform) = value;
        });

        self.controller.signal_pyramid_transform_update();
    }
}

#[component]
fn ScalarConfiguration(
    controller: ScalarController,
    #[prop(into)] title: String,
    #[prop(optional)] min: Option<f32>,
    #[prop(optional)] max: Option<f32>,
    #[prop(optional)] step: Option<f32>,
) -> impl IntoView {
    view! {
        <div style="display: flex; flex-direction: column; gap: 8px;">
            <label for=title.clone()>{title.clone()}</label>
            <div style="display: flex; gap: 16px; justify-content: space-between;">
                <input
                    id=title.clone()
                    type="number"
                    min=min
                    max=max
                    step=step
                    prop:value=controller.value()
                    on:change=move |_| controller.sync_engine_value()
                    on:input=move |event| {
                        // Set the new value
                        let new_value = event_target_value(&event);
                        controller.value().set(new_value.clone());

                        // Parse the input value.
                        let new_value = match new_value.parse::<f32>() {
                            Ok(new_value) => {
                                controller.err().set(None);
                                new_value.clamp(
                                    min.unwrap_or(f32::NEG_INFINITY),
                                    max.unwrap_or(f32::INFINITY),
                                )
                            }
                            Err(e) => {
                                controller.err().set(Some(e.to_string()));
                                return;
                            }
                        };

                        // Update the engine value.
                        controller.set_engine_value(new_value);
                    }
                />
                <button on:click=move |_| controller.reset()>
                    "Reset"
                </button>
            </div>
            <input
                id=title.clone()
                type="range"
                min=min
                max=max
                step=step
                prop:value=controller.value()
                on:change=move |_| controller.sync_engine_value()
                on:input=move |event| {
                    // Set the new value
                    let new_value = event_target_value(&event);
                    controller.value().set(new_value.clone());

                    // Parse the input value.
                    let new_value = match new_value.parse::<f32>() {
                        Ok(new_value) => {
                            controller.err().set(None);
                            new_value.clamp(
                                min.unwrap_or(f32::NEG_INFINITY),
                                max.unwrap_or(f32::INFINITY),
                            )
                        }
                        Err(e) => {
                            controller.err().set(Some(e.to_string()));
                            return;
                        }
                    };

                    // Update the engine value.
                    controller.set_engine_value(new_value);
                }
            />
            <Show when=move || controller.err().get().is_some()>
                <div style="color: red;">
                    {format!(
                        "{title} error: {err}",
                        err = controller.err().get().unwrap(),
                    )}
                </div>
            </Show>
        </div>
    }
}

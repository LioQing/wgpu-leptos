#[cfg(target_arch = "wasm32")]
/// Set a timeout in milliseconds.
pub fn set_timeout(callback: impl FnMut() + 'static, millis: i32) {
    use wasm_bindgen::{prelude::*, JsCast};

    let closure = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);
    web_sys::window()
        .expect("window")
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            millis,
        )
        .unwrap();
    closure.forget();
}

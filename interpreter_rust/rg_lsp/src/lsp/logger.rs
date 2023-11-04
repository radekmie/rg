use web_sys::wasm_bindgen::JsValue;

pub fn log(value: &JsValue) {
    web_sys::console::log_2(&"SERVER  ".into(), &value);
}
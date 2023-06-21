use wasm_bindgen::prelude::*;
use anyhow::Error as AnyhowErr;

#[inline]
pub fn err_to_js_err(err: AnyhowErr) -> JsError {
    JsError::new(&format!("{}", err))
}

#![feature(
    sync_unsafe_cell,
    iter_intersperse,
    mixed_integer_ops_unsigned_sub,
    let_chains
)]


// Utility
mod iter;

// Connection
mod ws;
mod state;

// UX
mod cover;
mod filetree;
mod filetabs;
mod code;


use std::panic;
use wasm_bindgen::prelude::*;


#[wasm_bindgen(start)]
pub fn start() {
    panic::set_hook(Box::new(|info| {
        error(&format!("{}\n{}", info, Error::new().stack()));
    }));
    filetree::init();
    code::init();
    ws::start();
}


#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(js_name = "setInterval")]
    fn set_interval(callback : &JsValue, duration_ms : u32);

}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(msg : &str);

    type Error;

    #[wasm_bindgen(constructor)]
    fn new() -> Error;

    #[wasm_bindgen(method, getter)]
    fn stack(this : &Error) -> String;
}


// TODO: Remove
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn warn(msg : &str);
    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    fn warnjs(msg : JsValue);
}

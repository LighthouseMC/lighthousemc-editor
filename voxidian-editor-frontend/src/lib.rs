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


use wasm_bindgen::prelude::*;


#[wasm_bindgen(start)]
pub fn start() {
    filetree::init();
    code::init();
    ws::start();
}


#[wasm_bindgen]
extern "C" {

    #[wasm_bindgen(js_name = "setInterval")]
    fn set_interval(callback : &JsValue, duration_ms : u32);

}


// TODO: Remove
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn warn(msg : &str);
    #[wasm_bindgen(js_namespace = console, js_name = warn)]
    fn warnjs(msg : JsValue);
}

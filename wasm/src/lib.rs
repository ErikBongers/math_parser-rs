//
//
// build me with : Command: build --target web --out-dir www



mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
//TODO:: implement sourcel list based parsing.
pub fn parse(startscript_id: String, mainscript_id: String) -> String {
    math_parser::parse("TODO".to_string())
}

#[wasm_bindgen]
pub fn parse_direct(text: String) -> String {
    math_parser::parse(text)
}

#[wasm_bindgen]
pub fn upload_source(script_id: String, text: String) -> i32 {
    math_parser::upload_source(text)
}

#[wasm_bindgen]
pub fn get_math_version() -> String {
    math_parser::get_math_version()
}


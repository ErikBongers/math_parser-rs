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
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, test_wasm!");
}

#[wasm_bindgen]
pub fn hello() -> String {
    "Hello, from Erik in main project. UPDATE".into()
}


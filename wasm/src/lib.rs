// build me with : Command: build --target web --out-dir www
mod utils;

use math_parser::Api;
use wasm_bindgen::prelude::*;
use crate::utils::set_panic_hook;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct MathParser {
    api: Api,
}

#[wasm_bindgen]
impl MathParser {
    pub fn new() -> MathParser {
        set_panic_hook();//make sure this is called only once.
        MathParser {
            api: Api::new(),
        }
    }

    pub fn add_source(&mut self, name: String, text: String) -> i32 {//TODO: rename upload_source...although...confusing as it could meand upload to cloud storage.
        self.api.set_source(name, text)
    }

    pub fn parse(&mut self, start_script_id: String, main_script_id: String) -> String {
        self.api.parse(start_script_id, main_script_id)
    }

    pub fn get_math_version() -> String {
        Api::get_math_version()
    }

}

#[wasm_bindgen]
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


// build me with : Command: build --target web --out-dir www
mod utils;

use math_parser::Api;
use wasm_bindgen::prelude::*;
use crate::utils::set_panic_hook;

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

    pub fn set_source(&mut self, name: String, text: String) -> i32 {
        self.api.set_source(name, text)
    }

    pub fn parse(&mut self, start_script_id: String, main_script_id: String) -> String {
        self.api.parse(start_script_id, main_script_id)
    }

    pub fn get_math_version() -> String {
        Api::get_math_version()
    }

}

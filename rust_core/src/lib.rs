use wasm_bindgen::prelude::*;

// This attribute exposes our Rust function to JavaScript.
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello from Rust, {}!", name)
}
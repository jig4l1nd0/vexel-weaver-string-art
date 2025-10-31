use wasm_bindgen::prelude::*;
use serde::Serialize;

// for numeric constants like PI:
use std::f64::consts::PI;

// The #[wasm_bindgen] attribute makes this enum usable from JavaScript.
#[wasm_bindgen]
pub enum Shape {
    Circle,
    Square,
}

// The #[derive(Serialize)] allows us to convert this struct into JSON.
// The #[wasm_bindgen] allows this struct to be understood by JS.
#[derive(Serialize, Clone, Copy)]
#[wasm_bindgen]
pub struct Pin {
    pub x: f64,
    pub y: f64,
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello from Rust, {}!", name)
}

// Our main function! It will generate the pin coordinates.
#[wasm_bindgen]
pub fn generate_pins(shape: Shape, num_pins: u32, width: f64, height: f64) -> Result<JsValue, JsValue> {
    let pins = match shape {
        Shape::Circle => generate_circular_pins(num_pins, width, height),
        Shape::Square => generate_square_pins(num_pins, width, height),
    };
    // Use serde_wasm_bindgen to convert the Rust Vec<Pin> into a JS Array of objects.
    Ok(serde_wasm_bindgen::to_value(&pins)?)
}

// --- Helper Functions (private to the Rust module) ---

fn generate_circular_pins(num_pins: u32, width: f64, height: f64) -> Vec<Pin> {
    let mut pins = Vec::new();
    let center_x = width / 2.0;
    let center_y = height / 2.0;
    let radius_x = width / 2.0;
    let radius_y = height / 2.0; // For ovals, but we'll use the smaller for a perfect circle

    let radius = radius_x.min(radius_y);

    for i in 0..num_pins {
        let angle = 2.0 * PI * (i as f64) / (num_pins as f64);
        let x = center_x + radius * angle.cos();
        let y = center_y + radius * angle.sin();
        pins.push(Pin { x, y });
    }
    pins
}

fn generate_square_pins(num_pins: u32, width: f64, height: f64) -> Vec<Pin> {
    let mut pins = Vec::new();
    let perimeter = 2.0 * (width + height);

    for i in 0..num_pins {
        let distance = perimeter * (i as f64) / (num_pins as f64);
        let mut x = 0.0;
        let mut y = 0.0;

        if distance < width { // Top edge
            x = distance;
            y = 0.0;
        } else if distance < width + height { // Right edge
            x = width;
            y = distance - width;
        } else if distance < 2.0 * width + height { // Bottom edge
            x = width - (distance - width - height);
            y = height;
        } else { // Left edge
            x = 0.0;
            y = height - (distance - 2.0 * width - height);
        }
        pins.push(Pin { x, y });
    }
    pins
}
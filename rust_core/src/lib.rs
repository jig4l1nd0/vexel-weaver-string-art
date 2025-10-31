use wasm_bindgen::prelude::*;
use serde::Serialize;
use serde::Deserialize;
use image::{
    load_from_memory, GrayImage, imageops, GenericImageView, DynamicImage
};
use std::f64::consts::PI;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use line_drawing::WalkGrid;

// --- Global State ---
// This holds our grayscale image data between function calls.
// The Mutex ensures we can safely modify it.
lazy_static! {
    static ref IMAGE_DATA: Mutex<Option<GrayImage>> = Mutex::new(None);
    static ref ORIGINAL_IMAGE: Mutex<Option<DynamicImage>> = Mutex::new(None); // Store original for processing
}

// --- Data Structures ---
#[wasm_bindgen]
pub enum Shape {
    Circle,
    Square,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[wasm_bindgen]
pub struct Pin {
    pub x: f64,
    pub y: f64,
}

// --- Public API Functions (callable from JS) ---

#[wasm_bindgen]
pub fn process_image(
    image_data: &[u8],
    canvas_width: u32,
    canvas_height: u32,
    zoom_level: f64,
    offset_x: f64,
    offset_y: f64,
) -> Result<(), JsValue> {
    // Attempt to load the original image data if it's not already stored
    if ORIGINAL_IMAGE.lock().is_none() {
        let image = load_from_memory(image_data).map_err(|e| e.to_string())?;
        *ORIGINAL_IMAGE.lock() = Some(image);
    }

    let original_image_guard = ORIGINAL_IMAGE.lock();
    let original_image = original_image_guard.as_ref().ok_or("Original image not loaded")?;
    
    let original_width = original_image.width() as f64;
    let original_height = original_image.height() as f64;

    // Calculate effective image dimensions after zoom
    let zoomed_width = original_width * zoom_level;
    let zoomed_height = original_height * zoom_level;

    // Calculate source rectangle in original image coordinates
    // These are the top-left (src_x, src_y) and bottom-right (src_x + crop_w, src_y + crop_h)
    // of the region we want to crop from the *zoomed* image, scaled back to original
    let crop_x_start_original = (-offset_x / zoom_level).max(0.0);
    let crop_y_start_original = (-offset_y / zoom_level).max(0.0);
    
    let crop_width_original = (canvas_width as f64 / zoom_level).min(original_width - crop_x_start_original);
    let crop_height_original = (canvas_height as f64 / zoom_level).min(original_height - crop_y_start_original);

    // Ensure crop dimensions are positive
    let crop_width_original = crop_width_original.max(1.0);
    let crop_height_original = crop_height_original.max(1.0);

    // Clamp crop start to valid image bounds
    let crop_x_start_original = crop_x_start_original.min(original_width - crop_width_original);
    let crop_y_start_original = crop_y_start_original.min(original_height - crop_height_original);
    
    // Crop the image
    let cropped_image = original_image.crop_imm(
        crop_x_start_original as u32, 
        crop_y_start_original as u32, 
        crop_width_original as u32, 
        crop_height_original as u32
    );

    // Resize the cropped image to fit the canvas dimensions
    let resized_image = cropped_image.resize_exact(canvas_width, canvas_height, imageops::FilterType::Triangle);
    
    // Convert to grayscale and store
    *IMAGE_DATA.lock() = Some(resized_image.to_luma8());
    
    Ok(())
}

#[wasm_bindgen]
pub fn generate_pins(shape: Shape, num_pins: u32, width: f64, height: f64) -> Result<JsValue, JsValue> {
    // This function remains the same
    let pins = match shape {
        Shape::Circle => generate_circular_pins(num_pins, width, height),
        Shape::Square => generate_square_pins(num_pins, width, height),
    };
    Ok(serde_wasm_bindgen::to_value(&pins)?)
}

#[wasm_bindgen]
pub fn generate_string_art(pins: JsValue, num_lines: u32) -> Result<JsValue, JsValue> {
    // 1. Get image and pin data
    let mut image_guard = IMAGE_DATA.lock();
    let image = image_guard.as_mut().ok_or("Image not loaded")?;
    let pins: Vec<Pin> = serde_wasm_bindgen::from_value(pins)?;
    
    if pins.is_empty() {
        return Err("No pins provided".into());
    }

    // 2. Main algorithm
    let mut line_sequence = Vec::with_capacity(num_lines as usize);
    let mut current_pin_index = 0;
    
    for _ in 0..num_lines {
        let mut best_next_pin_index = 0;
        let mut max_score = -1.0; // Use a float for scoring
        
        let start_pin = pins[current_pin_index];

        // Find the best line from the current pin
        for (i, end_pin) in pins.iter().enumerate() {
            if i == current_pin_index { continue; } // Don't connect a pin to itself

            let line_pixels = WalkGrid::new((start_pin.x as i32, start_pin.y as i32), (end_pin.x as i32, end_pin.y as i32));
            let mut current_score = 0.0;
            
            for (px, py) in line_pixels {
                if let Some(pixel) = image.get_pixel_checked(px as u32, py as u32) {
                    // Invert the value: darker pixels (lower value) give higher score
                    current_score += 255.0 - pixel[0] as f64;
                }
            }
            
            if current_score > max_score {
                max_score = current_score;
                best_next_pin_index = i;
            }
        }
        
        // 3. Update state
        line_sequence.push(best_next_pin_index);
        
        // "Erase" the chosen line from the image by making it lighter
        let best_pin = pins[best_next_pin_index];
        let line_to_erase = WalkGrid::new((start_pin.x as i32, start_pin.y as i32), (best_pin.x as i32, best_pin.y as i32));
        for (px, py) in line_to_erase {
            if let Some(pixel) = image.get_pixel_mut_checked(px as u32, py as u32) {
                // Add brightness, capping at 255 (white)
                pixel[0] = (pixel[0] as u16 + 150).min(255) as u8;
            }
        }
        
        current_pin_index = best_next_pin_index;
    }

    Ok(serde_wasm_bindgen::to_value(&line_sequence)?)
}


// --- Helper Functions (private) ---
fn generate_circular_pins(num_pins: u32, width: f64, height: f64) -> Vec<Pin> {
    // ... same code ...
    let mut pins = Vec::new();
    let center_x = width / 2.0;
    let center_y = height / 2.0;
    let radius = (width / 2.0).min(height / 2.0);

    for i in 0..num_pins {
        let angle = 2.0 * PI * (i as f64) / (num_pins as f64);
        let x = center_x + radius * angle.cos();
        let y = center_y + radius * angle.sin();
        pins.push(Pin { x, y });
    }
    pins
}

fn generate_square_pins(num_pins: u32, width: f64, height: f64) -> Vec<Pin> {
    // ... same code ...
    let mut pins = Vec::new();
    let perimeter = 2.0 * (width + height);

    for i in 0..num_pins {
        let distance = perimeter * (i as f64) / (num_pins as f64);
        let mut x = 0.0;
        let mut y = 0.0;

        if distance < width {
            x = distance;
            y = 0.0;
        } else if distance < width + height {
            x = width;
            y = distance - width;
        } else if distance < 2.0 * width + height {
            x = width - (distance - width - height);
            y = height;
        } else {
            x = 0.0;
            y = height - (distance - 2.0 * width - height);
        }
        pins.push(Pin { x, y });
    }
    pins
}
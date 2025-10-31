import init, { generate_pins, Shape } from '../rust_core/pkg/rust_core.js';

// A dedicated function to handle drawing
function drawPins(ctx, pins) {
    ctx.fillStyle = 'black'; // Set the color for the pins

    for (const pin of pins) {
        ctx.beginPath(); // Start a new shape
        // Draw a circle: arc(x, y, radius, startAngle, endAngle)
        ctx.arc(pin.x, pin.y, 2, 0, 2 * Math.PI);
        ctx.fill(); // Fill the circle with the current fillStyle
    }
}

async function run() {
    // Initialize the wasm module
    await init();

    // Get the canvas and its 2D rendering context
    const canvas = document.getElementById('art-canvas');
    const ctx = canvas.getContext('2d');

    // Define parameters
    const num_pins = 200;
    const width = canvas.width;
    const height = canvas.height;
    
    // Generate pins for a square using our Rust function
    const pins = generate_pins(Shape.Square, num_pins, width, height);
    
    // Draw them!
    drawPins(ctx, pins);
    
    // You can uncomment the lines below to see the circle pattern instead
    // ctx.clearRect(0, 0, width, height); // Clear the canvas first
    // const circle_pins = generate_pins(Shape.Circle, num_pins, width, height);
    // drawPins(ctx, circle_pins);
}

run();
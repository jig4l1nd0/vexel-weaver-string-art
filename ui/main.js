// Import all our Rust functions
import init, {
    generate_pins,
    Shape,
    process_image, // Renamed from load_image
    generate_string_art
} from '../rust_core/pkg/rust_core.js';

// --- Draw Functions (remain the same) ---
function drawPins(ctx, pins) {
    ctx.fillStyle = 'black';
    for (const pin of pins) {
        ctx.beginPath();
        ctx.arc(pin.x, pin.y, 2, 0, 2 * Math.PI);
        ctx.fill();
    }
}

// Draw the final string art lines
function drawLines(ctx, pins, line_sequence) {
    ctx.strokeStyle = "rgba(0, 0, 0, 0.1)"; // Recommend starting with 0.05
    ctx.lineWidth = 2.0;
    
    // The current pin index
    let current_pin_index = 0; 

    // Iterate over each LINE in the sequence
    for (const next_pin_index of line_sequence) {
        
        // Get the coordinates of the START and END pins
        const start_pin = pins[current_pin_index];
        const end_pin = pins[next_pin_index];

        // 1. Start a NEW path for EACH line
        ctx.beginPath(); 
        
        // 2. Move to the start of this line
        ctx.moveTo(start_pin.x, start_pin.y); 
        
        // 3. Draw the segment
        ctx.lineTo(end_pin.x, end_pin.y);
        
        // 4. Stroke THIS SINGLE LINE
        ctx.stroke(); 

        // 5. Update the current pin for the next iteration
        current_pin_index = next_pin_index;
    }
}

// --- Image Preview Drawing ---
// This function draws the currently uploaded image on the preview canvas
function drawPreviewImage(ctx, originalImg, zoom, offsetX, offsetY, canvasWidth, canvasHeight) {
    ctx.clearRect(0, 0, canvasWidth, canvasHeight); // Clear the preview canvas
    if (!originalImg) return;

    // Calculate dimensions of the image when zoomed
    const imgWidth = originalImg.width * zoom;
    const imgHeight = originalImg.height * zoom;

    // Draw the image, applying zoom and offset
    ctx.drawImage(originalImg, offsetX, offsetY, imgWidth, imgHeight);
}


// --- Main Application Logic ---
async function run() {
    await init();

    // Get UI elements
    const artCanvas = document.getElementById('art-canvas');
    const artCtx = artCanvas.getContext('2d');
    const previewCanvas = document.getElementById('preview-canvas');
    const previewCtx = previewCanvas.getContext('2d');

    const imageUpload = document.getElementById('image-upload');
    const uploadStatus = document.getElementById('upload-status');
    const numLinesInput = document.getElementById('num-lines');
    const numPinsInput = document.getElementById('num-pins');
    const shapeSelect = document.getElementById('shape-select');
    const generateButton = document.getElementById('generate-button');
    const resetViewButton = document.getElementById('reset-view-button');

    // --- Global State ---
    let pins = [];
    let imageLoaded = false;
    let originalImageData = null; // Raw bytes of the uploaded image
    let currentImageElement = null; // <img/> element for preview canvas drawing
    let zoomLevel = 1.0;
    let offsetX = 0;
    let offsetY = 0;
    let isDragging = false;
    let lastMouseX = 0;
    let lastMouseY = 0;

    const canvasWidth = artCanvas.width;
    const canvasHeight = artCanvas.height;

    // --- Helper to redraw everything on preview canvas ---
    const redrawPreview = () => {
        drawPreviewImage(previewCtx, currentImageElement, zoomLevel, offsetX, offsetY, canvasWidth, canvasHeight);
        drawPins(previewCtx, pins); // Optionally draw pins on preview
    };

    // --- Step 0: Initial setup ---
    // Generate and draw the initial pins for the art canvas
    pins = generate_pins(Shape.Circle, parseInt(numPinsInput.value, 10), canvasWidth, canvasHeight);
    drawPins(artCtx, pins);
    drawPins(previewCtx, pins); // Also draw on preview initially

    // --- Step 1: Image Upload ---
    imageUpload.addEventListener('change', (event) => {
        const file = event.target.files[0];
        if (!file) return;

        uploadStatus.textContent = "Loading...";
        const reader = new FileReader();

        reader.onload = (e) => {
            originalImageData = new Uint8Array(e.target.result);
            
            // Create an Image object for the preview canvas
            const img = new Image();
            img.onload = () => {
                currentImageElement = img;
                // Reset view parameters for new image
                zoomLevel = 1.0;
                offsetX = 0;
                offsetY = 0;
                redrawPreview(); // Draw image to preview canvas
                
                imageLoaded = true;
                generateButton.disabled = false;
                resetViewButton.disabled = false;
                uploadStatus.textContent = "Image loaded!";
            };
            img.onerror = () => {
                uploadStatus.textContent = "Error loading image for preview.";
                currentImageElement = null;
            };
            img.src = URL.createObjectURL(file); // Load image for HTML Image object
        };
        reader.readAsArrayBuffer(file); // Read as ArrayBuffer for Rust
    });

    // --- Image Preview Interaction (Zoom & Pan) ---
    previewCanvas.addEventListener('wheel', (e) => {
        if (!imageLoaded) return;
        e.preventDefault(); // Prevent page scrolling

        const scaleAmount = 1.1; // Zoom factor
        const mouseX = e.offsetX; // Mouse position relative to canvas
        const mouseY = e.offsetY;

        // Calculate new zoom level
        const newZoom = e.deltaY < 0 ? zoomLevel * scaleAmount : zoomLevel / scaleAmount;

        // Don't let zoom go too low
        if (newZoom < 0.1) return;

        // Adjust offsets to keep the mouse pointer's position relative to the image consistent
        offsetX = mouseX - ((mouseX - offsetX) * (newZoom / zoomLevel));
        offsetY = mouseY - ((mouseY - offsetY) * (newZoom / zoomLevel));
        zoomLevel = newZoom;
        
        redrawPreview();
    });

    previewCanvas.addEventListener('mousedown', (e) => {
        if (!imageLoaded) return;
        isDragging = true;
        lastMouseX = e.offsetX;
        lastMouseY = e.offsetY;
        previewCanvas.classList.add('grabbing'); // Change cursor
    });

    previewCanvas.addEventListener('mousemove', (e) => {
        if (!imageLoaded || !isDragging) return;
        
        const dx = e.offsetX - lastMouseX;
        const dy = e.offsetY - lastMouseY;

        offsetX += dx;
        offsetY += dy;

        lastMouseX = e.offsetX;
        lastMouseY = e.offsetY;
        
        redrawPreview();
    });

    previewCanvas.addEventListener('mouseup', () => {
        isDragging = false;
        previewCanvas.classList.remove('grabbing');
    });

    previewCanvas.addEventListener('mouseout', () => { // Stop dragging if mouse leaves canvas
        isDragging = false;
        previewCanvas.classList.remove('grabbing');
    });

    resetViewButton.addEventListener('click', () => {
        if (!imageLoaded) return;
        zoomLevel = 1.0;
        offsetX = 0;
        offsetY = 0;
        redrawPreview();
    });

    // --- Parameter Changes (Pins & Shape) ---
    numPinsInput.addEventListener('change', () => {
        const shapeType = shapeSelect.value === 'circle' ? Shape.Circle : Shape.Square;
        pins = generate_pins(shapeType, parseInt(numPinsInput.value, 10), canvasWidth, canvasHeight);
        drawPins(artCtx, pins);
        redrawPreview();
    });

    shapeSelect.addEventListener('change', () => {
        const shapeType = shapeSelect.value === 'circle' ? Shape.Circle : Shape.Square;
        pins = generate_pins(shapeType, parseInt(numPinsInput.value, 10), canvasWidth, canvasHeight);
        drawPins(artCtx, pins);
        redrawPreview();
    });


    // --- Step 3: Generate Art ---
    generateButton.addEventListener('click', () => {
        if (!imageLoaded) {
            alert("Please upload an image first.");
            return;
        }

        console.log("Generating art...");
        artCtx.clearRect(0, 0, canvasWidth, canvasHeight); // Clear art canvas
        
        const num_lines = parseInt(numLinesInput.value, 10);
        
        // --- THIS IS THE KEY CALL ---
        // Process the image with current zoom and offset before generating art
        process_image(originalImageData, canvasWidth, canvasHeight, zoomLevel, offsetX, offsetY);
        
        const line_sequence = generate_string_art(pins, num_lines);

        console.log("Drawing final art...");
        drawLines(artCtx, pins, line_sequence);
        drawPins(artCtx, pins); // Redraw pins on top (optional)
        console.log("Done!");
    });
}

run();
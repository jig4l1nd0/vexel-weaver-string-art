// Import the functions from the wasm package
import init, { greet } from '../rust_core/pkg/rust_core.js';

async function run() {
  // Initialize the wasm module
  await init();

  // Call our exported Rust function
  const message = greet("Web");
  alert(message);
}

run();
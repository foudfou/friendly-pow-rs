#!/bin/bash
set -e

echo "🦀 Building Friendly POW Rust/WASM..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Build for web
echo "📦 Building WASM module..."
wasm-pack build --target web --release

# Run tests
echo "🧪 Running tests..."
cargo test

echo "✅ Build complete!"
echo ""
echo "To test the demo:"
echo "  cd examples"
echo "  python3 -m http.server 8080"
echo "  # Open http://localhost:8080/demo.html"

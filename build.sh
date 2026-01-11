#!/bin/bash
set -e

echo "🦀 Building Friendly POW Rust/WASM..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found. Installing..."
    cargo install wasm-pack
fi

# Build standard version
echo "📦 Building standard WASM module..."
wasm-pack build --target web --release --out-dir pkg

# Build SIMD-enabled version
echo "📦 Building SIMD-enabled WASM module..."
RUSTFLAGS='-C target-feature=+simd128' wasm-pack build --target web --release --out-dir pkg-simd

# Run tests
echo "🧪 Running tests..."
cargo test

echo "✅ Build complete!"
echo ""
echo "Packages built:"
echo "  ./pkg - Standard WASM (compatible with all browsers)"
echo "  ./pkg-simd - SIMD-enabled WASM (requires Chrome 91+, Firefox 89+, Safari 16.4+)"
echo ""
echo "Note: Both currently use optimized Blake2b with state reuse."
echo "True WASM SIMD speedup would require implementing wasm32 intrinsics."
echo ""
echo "To test the demo:"
echo "  python3 -m http.server 8080"
echo "  # Open http://localhost:8080/examples/demo.html"
echo ""
echo "To run native benchmarks:"
echo "  cargo run --example benchmark --release"

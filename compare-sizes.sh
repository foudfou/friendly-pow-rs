#!/bin/bash
set -e

echo "🦀 Building all variants to compare WASM sizes..."
echo ""

# Clean previous builds
rm -rf pkg-* size-comparison.txt

# Build basic variant (only basic solver)
echo "📦 Building Basic variant..."
wasm-pack build --target web --release --out-dir pkg-basic -- --no-default-features --features variant-basic 2>&1 | grep -E "(Compiling|Finished|Done)" || true

# Build optimized variant (only optimized solver)
echo "📦 Building Optimized variant..."
wasm-pack build --target web --release --out-dir pkg-optimized -- --no-default-features --features variant-optimized 2>&1 | grep -E "(Compiling|Finished|Done)" || true

# Build SIMD variant (optimized + SIMD flags)
echo "📦 Building SIMD variant..."
RUSTFLAGS='-C target-feature=+simd128' wasm-pack build --target web --release --out-dir pkg-simd -- --no-default-features --features variant-simd 2>&1 | grep -E "(Compiling|Finished|Done)" || true

# Build all variants together (default - for comparison)
echo "📦 Building All variants together..."
wasm-pack build --target web --release --out-dir pkg-all -- --all-features 2>&1 | grep -E "(Compiling|Finished|Done)" || true

echo ""
echo "✅ All builds complete!"
echo ""
echo "========================================="
echo "WASM Size Comparison"
echo "========================================="

# Function to get file size in bytes and KB
get_size() {
    local file=$1
    if [ -f "$file" ]; then
        local bytes=$(wc -c < "$file")
        local kb=$(echo "scale=2; $bytes / 1024" | bc)
        echo "$bytes bytes ($kb KB)"
    else
        echo "N/A"
    fi
}

# Compare sizes
echo ""
echo "Variant             | WASM Size"
echo "--------------------|-----------------"
echo "Basic only          | $(get_size pkg-basic/friendly_pow_rs_bg.wasm)"
echo "Optimized only      | $(get_size pkg-optimized/friendly_pow_rs_bg.wasm)"
echo "SIMD (with +simd128)| $(get_size pkg-simd/friendly_pow_rs_bg.wasm)"
echo "All variants        | $(get_size pkg-all/friendly_pow_rs_bg.wasm)"
echo ""

# Calculate differences
if [ -f "pkg-basic/friendly_pow_rs_bg.wasm" ] && [ -f "pkg-optimized/friendly_pow_rs_bg.wasm" ]; then
    basic_size=$(wc -c < pkg-basic/friendly_pow_rs_bg.wasm)
    optimized_size=$(wc -c < pkg-optimized/friendly_pow_rs_bg.wasm)
    simd_size=$(wc -c < pkg-simd/friendly_pow_rs_bg.wasm)
    all_size=$(wc -c < pkg-all/friendly_pow_rs_bg.wasm)

    opt_diff=$(($optimized_size - $basic_size))
    simd_diff=$(($simd_size - $optimized_size))
    all_diff=$(($all_size - $basic_size))

    echo "Size differences:"
    echo "  Optimized vs Basic:  $(printf '%+d' $opt_diff) bytes (custom Blake2b overhead)"
    echo "  SIMD vs Optimized:   $(printf '%+d' $simd_diff) bytes (SIMD flags impact)"
    echo "  All vs Basic:        $(printf '%+d' $all_diff) bytes (all variants together)"
    echo ""
fi

# Also show JS wrapper sizes
echo "JavaScript wrapper sizes:"
echo "  Basic:     $(get_size pkg-basic/friendly_pow_rs.js)"
echo "  Optimized: $(get_size pkg-optimized/friendly_pow_rs.js)"
echo "  SIMD:      $(get_size pkg-simd/friendly_pow_rs.js)"
echo "  All:       $(get_size pkg-all/friendly_pow_rs.js)"
echo ""

echo "Note: 'All variants' includes all three solver methods in one WASM file."
echo "This is what the default build produces."

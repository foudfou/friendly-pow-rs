# Solver Variants

This project implements three solver variants for performance comparison and exploration.

## Variants

### 1. Basic (`solve_blake2b`)
- **Description**: Original implementation using RustCrypto's blake2 crate
- **Approach**: Creates new Blake2b hasher for each iteration
- **Pros**: Simple, uses well-tested library
- **Cons**: Slower due to repeated initialization
- **Use case**: Baseline reference, maximum compatibility

### 2. Optimized (`solve_blake2b_optimized`)
- **Description**: Port of AssemblyScript optimization with state reuse
- **Approach**: Creates Blake2b context once, resets only hash state between iterations
- **Key optimization**: `reset_for_short_message()` - resets h[] vector to IV without clearing buffer
- **Pros**: ~50% faster in WASM (based on AssemblyScript benchmarks)
- **Use case**: Recommended for production WASM use
- **Implementation**: Custom Blake2b in `src/blake2b_custom.rs`

### 3. SIMD (`solve_blake2b_simd`)
- **Description**: WASM SIMD-optimized build using wasm32 intrinsics
- **Approach**: Uses `std::arch::wasm32::v128` and `i64x2_*` intrinsics for vectorized operations
- **Build flags**: `RUSTFLAGS='-C target-feature=+simd128'`
- **Browser support**: Chrome 91+, Firefox 89+, Safari 16.4+
- **SIMD operations**:
  - `v128_load` for loading message blocks (2 u64s at once)
  - `i64x2_xor` for final state mixing
  - `i64x2` for creating SIMD vectors
- **Implementation**: Custom Blake2b in `src/blake2b_simd.rs`

## Performance Characteristics

### Native (x86_64)
The optimized variant may be **slower** on native targets because:
- RustCrypto's blake2 crate uses highly optimized native SIMD (AVX2, SSE)
- Our custom implementation is optimized for WASM, not native

### WebAssembly
The optimized variant should be **faster** in WASM because:
- WASM doesn't have the same aggressive optimizations as native
- State reuse avoids repeated initialization overhead
- This is where the AssemblyScript version showed 50% speedup

## Building

```bash
# Build both standard and SIMD packages
./build.sh

# Or manually:
wasm-pack build --target web --out-dir pkg
RUSTFLAGS='-C target-feature=+simd128' wasm-pack build --target web --out-dir pkg-simd
```

## Testing

### Browser Testing (Recommended)
```bash
python3 -m http.server 8080
# Open http://localhost:8080/examples/demo.html
# Select variant from dropdown and compare performance
```

### Native Benchmarks
```bash
cargo run --example benchmark --release
```

Note: Native benchmarks verify correctness but don't reflect WASM performance.

## Implementation Details

### Custom Blake2b (`src/blake2b_custom.rs`)

Based on friendly-pow's `blake2b64.ts`, our implementation:
- Exposes internal state (h[] vector)
- Implements `reset_for_short_message()` for state-only reset
- Uses u64 arithmetic matching the AssemblyScript version
- Verified to produce identical output to RustCrypto's blake2

Key functions:
- `Blake2bContext::new(outlen)` - Create context
- `reset_for_short_message()` - Reset hash state only (optimization!)
- `compress(&buffer, is_last)` - Compress 128-byte block
- `hash_value_u32()` - Get first 4 bytes as u32 (for threshold check)
- `finalize()` - Get final hash bytes

### Why State Reuse Works

For Friendly POW puzzles:
1. Input is always exactly 128 bytes
2. Only last 4 bytes (nonce) change between attempts
3. Blake2b processes in 128-byte blocks
4. We can keep the buffer in-place and only reset the hash state

This avoids:
- Reallocating/clearing buffers
- Reinitializing IV mixing
- Unnecessary memory operations

## Size Comparison

After building all variants (from `./compare-sizes.sh`):

| Variant | WASM Size | JS Wrapper | Notes |
|---------|-----------|------------|-------|
| Basic only | 26.92 KB | 8.92 KB | Uses blake2 crate |
| Optimized only | 22.40 KB | 9.10 KB | **16.8% smaller!** Custom Blake2b |
| SIMD (with +simd128) | 24.74 KB | 10.52 KB | True WASM SIMD intrinsics |
| All variants | 32.72 KB | 11.84 KB | All three methods together |

**Key findings:**
- Optimized is **smaller** than Basic (custom implementation is minimal)
- SIMD adds ~2.3 KB over Optimized (SIMD instruction overhead)
- All variants together adds only 5.9 KB vs Basic (good code sharing)

## SIMD Implementation Details

The SIMD variant (`src/blake2b_simd.rs`) uses true WASM SIMD:

```rust
use std::arch::wasm32::*;

// Load message blocks using SIMD (2 u64s at once)
let v = v128_load(buf_ptr.add(i * 16) as *const v128);
m[i * 2] = i64x2_extract_lane::<0>(v) as u64;
m[i * 2 + 1] = i64x2_extract_lane::<1>(v) as u64;

// XOR final state using SIMD vectors
let h_vec = i64x2(self.h[i * 2] as i64, self.h[i * 2 + 1] as i64);
let v_low = i64x2(v[i * 2] as i64, v[i * 2 + 1] as i64);
let v_high = i64x2(v[i * 2 + 8] as i64, v[i * 2 + 9] as i64);
let result = v128_xor(v128_xor(h_vec, v_low), v_high);
```

## Expected Performance

- **Basic variant**: Baseline performance
- **Optimized variant**: ~50% faster than basic (WASM), based on AssemblyScript results
- **SIMD variant**: Additional 20-40% faster than optimized (estimated, needs browser testing)

## References

- Original friendly-pow: `~/src/friendly-pow`
- AssemblyScript solver: `src/solverWasm.ts`
- AssemblyScript Blake2b: `src/blake2b/blake2b64.ts`
- Key insight: Line 48-49 of `solverWasm.ts` shows state reuse pattern

# 🦀 Friendly POW - Rust/WASM Implementation

A high-performance Rust/WebAssembly implementation of the Friendly Captcha Proof-of-Work (POW) solver, compatible with the original AssemblyScript version.

## Context

The motivation is nothing more than exploring
[friendly-pow](https://github.com/FriendlyCaptcha/friendly-pow), WASM and a
rust port, since rust apparently has become the de facto source language for
WASM as of 2025.

The rust version seems faster but what is the motivation to do POW in
WASM in the first place? In comparison,
[Anubis](https://github.com/TecharoHQ/anubis/blob/main/web/js/worker/sha256-webcrypto.ts)
uses plain JS/WebCrypto SHA-256.

There's probably a tradeoff between speed and friction for bots. So, as is, the
rust version may be lowering the friction. Friendly Captcha has means to tune
the difficulty by requiring more solutions to a challenge (`n` parameter):

> One could get lucky or unlucky in how many attempts are required to find a
> solution. In order to reduce the variance and allow us to show a progress bar
> to the user we actually have the client find multiple solutions (with a lower
> difficulty threshold).

Another noticeable difference is FC is seemingly stateless: each challenge is
signed, the signature is attached to the challenge and the client is expected
to send it back with the solution. Anubis on the other hand is stateful: no
signature attached to the challenge, but issued challenges are stored on the
backend and validated on reception. Pre-compute attacks are prevented because
of the huge (64 bytes) random nonce.

BTW early 2024 FC introduced
[v2](https://github.com/FriendlyCaptcha/friendly-captcha-sdk) which now relies
on *signal collection* (mouse movements etc). Apparently POW has been abandoned
in favor of behavioral analysis.

## How It Works

The solver finds a nonce that, when hashed with Blake2b-256, produces a hash value below a given threshold:

1. Takes a puzzle buffer (32-64 bytes, padded to 128)
2. Tries different nonce values in the last 4 bytes
3. Hashes the entire buffer with Blake2b-256
4. Checks if the first 4 bytes of the hash (as u32) < threshold
5. Returns the hash if solution found

### Difficulty Formula

```
Threshold = floor(2^((255.999 - difficulty) / 8))
```

- Higher difficulty → lower threshold → more work required
- Every 8 difficulty units ≈ one more leading zero bit needed
- Difficulty range: 0-255

### Expected Attempts

| Difficulty | Threshold | Expected Attempts | Time (approx) |
|------------|-----------|-------------------|---------------|
| 50         | ~59M      | ~73               | <1ms          |
| 120        | ~131K     | ~33K              | ~10ms         |
| 150        | ~8K       | ~524K             | ~150ms        |
| 175        | ~362      | ~11.8M            | ~4s           |
| 200        | ~128      | ~33.5M            | ~10s          |

## Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install wasm-pack
cargo install wasm-pack
```

## Building

### Quick build

```bash
./build.sh
```

### Manual build

```bash
# Build WASM module for web
wasm-pack build --target web --release

# Run tests
cargo test

# Build for Node.js
wasm-pack build --target nodejs --release

# Build for bundlers (webpack, rollup)
wasm-pack build --target bundler --release
```

## Usage

See `demo.html` and `node.js` [examples](./examples).

Note in a real world scenario the solver needs to run inside a web worker. FC
actually uses a
[WorkerGroup](https://github.com/FriendlyCaptcha/friendly-challenge/blob/master/src/workergroup.ts)
to distribute puzzles.

## Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapabilities

# Run WASM tests (requires wasm-pack)
wasm-pack test --headless --firefox
```

## Demo

1. Build the project:
   ```bash
   ./build.sh
   ```

2. Serve the demo:
   ```bash
   python3 -m http.server 8080
   ```

3. Open http://localhost:8080/examples/demo.html

## Performance Comparison

Benchmarks on a modern desktop (your results may vary):

| Implementation | Hash Rate | Notes                   |
|----------------|-----------|-------------------------|
| AssemblyScript | ~70 KH/s  | Original implementation |
| Rust (basic)   | ~150 MH/s | This implementation     |

The `simd` git branch explores optimizations but the `blake2` crate is actually
highly optimized already.

## License

This implementation is provided for educational purposes. The original Friendly Captcha uses a source-available license.

## Contributing

Contributions welcome! Areas for improvement:

- [ ] SIMD optimization with blake2b_simd
- [ ] Web Worker integration
- [ ] Benchmark suite
- [ ] Bundle size optimization
- [ ] Multi-threading support

## References

- [Friendly Captcha](https://friendlycaptcha.com/)
- [Original friendly-pow (AssemblyScript)](https://github.com/FriendlyCaptcha/friendly-pow)
- [Blake2 RFC](https://www.blake2.net/)
- [WebAssembly](https://webassembly.org/)

## Acknowledgments

Based on the Friendly Captcha POW algorithm, which itself draws from:
- Hashcash (Adam Back, 1997)
- Bitcoin's proof-of-work
- Blake2b cryptographic hash function

# Quick Start Guide

## 1. Install Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install wasm-pack
cargo install wasm-pack
```

## 2. Build the Project

```bash
cd /home/foudil/src/rs/friendly-pow-rs

# Quick build (runs tests and builds WASM)
./build.sh

# Or manually:
cargo test          # Run tests
wasm-pack build --target web --release   # Build WASM
```

## 3. Run the Demo

```bash
# Start a local web server
cd examples
python3 -m http.server 8080

# Open your browser to:
# http://localhost:8080/demo.html
```

## 4. Run Benchmark

```bash
cargo run --release --example benchmark
```

## 5. Use in Your Project

### For Web (ES Modules)

```javascript
import init, { Solver, difficulty_to_threshold } from './pkg/friendly_pow_rs.js';

await init();
const solver = new Solver();
const threshold = difficulty_to_threshold(120);
const hash = solver.solve_blake2b(puzzle, threshold, 100_000_000);
```

### For Node.js

```bash
wasm-pack build --target nodejs
```

```javascript
const { Solver, difficulty_to_threshold } = require('./pkg/friendly_pow_rs');
```

### For Webpack/Bundlers

```bash
wasm-pack build --target bundler
```

```javascript
import { Solver, difficulty_to_threshold } from 'friendly-pow-rs';
```

## 6. Understanding the Output

After building, you'll have a `pkg/` directory containing:

- `friendly_pow_rs.js` - JavaScript bindings
- `friendly_pow_rs_bg.wasm` - WebAssembly binary
- `friendly_pow_rs.d.ts` - TypeScript definitions
- `package.json` - NPM package metadata

## Troubleshooting

### wasm-pack not found
```bash
cargo install wasm-pack
```

### Permission denied on build.sh
```bash
chmod +x build.sh
```

### CORS errors in browser
Make sure you're serving the files via HTTP, not opening them directly as `file://`

### Tests failing
```bash
cargo clean
cargo test
```

## Next Steps

- Read the [README.md](README.md) for detailed documentation
- Check out [examples/demo.html](examples/demo.html) for browser usage
- Run `cargo doc --open` to view the API documentation
- Modify difficulty in the demo to see performance differences

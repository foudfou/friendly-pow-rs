// Example: Using friendly-pow-rs in Node.js
// Run: node examples/node.js
// Build first: wasm-pack build --target nodejs --release --out-dir pkg-nodejs

const { Solver, difficulty_to_threshold } = require('../pkg-nodejs/friendly_pow_rs.js');

console.log('🦀 Friendly POW - Node.js Example\n');

const solver = new Solver();
const puzzle = new Uint8Array([1, 2, 3, 4, 5]); // Buffer.from([1, 2, 3, 4, 5]);
const difficulty = 120;
const threshold = difficulty_to_threshold(difficulty);

console.log(`Difficulty: ${difficulty}`);
console.log(`Threshold: ${threshold}`);
console.log('Solving...\n');

const startTime = Date.now();
const hash = solver.solve_blake2b(puzzle, threshold, 100_000_000);
const endTime = Date.now();

if (hash.length > 0) {
    console.log('✅ Solution found!');
    console.log(`Hash: ${Buffer.from(hash).toString('hex')}`);
    console.log(`Time: ${endTime - startTime}ms`);
    console.log(`Nonce: ${Buffer.from(solver.get_solution_nonce()).toString('hex')}`);
} else {
    console.log('❌ No solution found');
}

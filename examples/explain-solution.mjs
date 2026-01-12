// Detailed explanation of how the POW solution works
import { Solver, difficulty_to_threshold } from '../pkg-nodejs/friendly_pow_rs.js';

console.log('📚 Friendly POW - Solution Explanation\n');
console.log('═'.repeat(60));

// Step 1: Input puzzle
const puzzle = new Uint8Array([1, 2, 3, 4, 5]);
console.log('\n📥 STEP 1: Input Puzzle');
console.log('Original puzzle bytes:', Array.from(puzzle).map(b => `0x${b.toString(16).padStart(2, '0')}`).join(', '));
console.log('Length:', puzzle.length, 'bytes');
console.log('Note: This will be padded to 128 bytes with zeros');

// Step 2: Difficulty and threshold
const difficulty = 120;
const threshold = difficulty_to_threshold(difficulty);
console.log('\n🎯 STEP 2: Difficulty → Threshold');
console.log('Difficulty:', difficulty);
console.log('Threshold:', threshold, `(0x${threshold.toString(16).padStart(8, '0')})`);
console.log('Formula: T = floor(2^((255.999 - d) / 8))');
const exponent = (255.999 - difficulty) / 8;
console.log(`         T = floor(2^((255.999 - ${difficulty}) / 8))`);
console.log(`         T = floor(2^${exponent.toFixed(6)})`);
console.log(`         T = ${threshold}`);

// Calculate expected leading zeros
const thresholdBits = threshold.toString(2);
const leadingZeros = 32 - thresholdBits.length;
console.log('\nThreshold in binary:', '0'.repeat(leadingZeros) + thresholdBits);
console.log('Leading zeros needed: ~', leadingZeros, 'bits');
console.log('Expected attempts: ~', Math.floor(Math.pow(2, 32) / threshold).toLocaleString());

// Step 3: Solve
console.log('\n⚙️  STEP 3: Solving (trying different nonces)...');
const solver = new Solver();
const hash = solver.solve_blake2b(puzzle, threshold, 100_000_000);

if (hash.length === 0) {
    console.log('❌ No solution found');
    process.exit(1);
}

// Step 4: Extract solution
const solution = solver.get_solution();
const nonce = solver.get_solution_nonce();

console.log('\n✅ STEP 4: Solution Found!');
console.log('\nFull 128-byte buffer (with nonce):');
console.log('First 5 bytes (original puzzle):', Array.from(solution.slice(0, 5)).map(b => `0x${b.toString(16).padStart(2, '0')}`).join(', '));
console.log('Bytes 5-123 (padding):          [all zeros]');
console.log('Last 8 bytes (nonce region):   ', Array.from(solution.slice(120, 128)).map(b => `0x${b.toString(16).padStart(2, '0')}`).join(' '));
console.log('\nNonce (last 8 bytes):', Buffer.from(nonce).toString('hex'));
console.log('Active nonce (last 4 bytes):', Buffer.from(solution.slice(124, 128)).toString('hex'));

console.log('\nVerify on the command-line with: printf "01020304050""0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000""' + Buffer.from(nonce).toString('hex') + '" | xxd -r -p | b2sum -l 256')

// Step 5: Verify the hash
console.log('\n🔍 STEP 5: Verify Solution');
console.log('\nHash (32 bytes):', Buffer.from(hash).toString('hex'));

// Extract first 4 bytes as little-endian u32
const hashBytes = new Uint8Array(hash);
const hashValue = hashBytes[0] | (hashBytes[1] << 8) | (hashBytes[2] << 16) | (hashBytes[3] << 24);

console.log('\nFirst 4 bytes of hash (little-endian):');
console.log('  Bytes:', `${hashBytes[0].toString(16).padStart(2, '0')} ${hashBytes[1].toString(16).padStart(2, '0')} ${hashBytes[2].toString(16).padStart(2, '0')} ${hashBytes[3].toString(16).padStart(2, '0')}`);
console.log('  As u32 (little-endian):', hashValue, `(0x${hashValue.toString(16).padStart(8, '0')})`);

// Count leading zeros in hash value
const hashBits = hashValue.toString(2);
const hashLeadingZeros = 32 - hashBits.length;
console.log('  In binary:', '0'.repeat(hashLeadingZeros) + hashBits);
console.log('  Leading zero bits:', hashLeadingZeros);

// Verify
console.log('\n📊 VALIDATION:');
console.log(`  Hash value:  ${hashValue.toLocaleString().padStart(10)} (0x${hashValue.toString(16).padStart(8, '0')})`);
console.log(`  Threshold:   ${threshold.toLocaleString().padStart(10)} (0x${threshold.toString(16).padStart(8, '0')})`);
console.log(`  Valid?       ${hashValue} < ${threshold} → ${hashValue < threshold ? '✅ YES' : '❌ NO'}`);

console.log('\n💡 SUMMARY:');
console.log(`  We found a nonce that when hashed produces ${hashLeadingZeros} leading zero bits,`);
console.log(`  which gives a hash value (${hashValue}) below the threshold (${threshold}).`);
console.log(`  This proves we did the computational work!`);
console.log('\n' + '═'.repeat(60));

use wasm_bindgen::prelude::*;

#[cfg(feature = "variant-basic")]
use blake2::{Blake2b, Digest};
#[cfg(feature = "variant-basic")]
use blake2::digest::consts::U32;

#[cfg(feature = "variant-optimized")]
mod blake2b_custom;
#[cfg(feature = "variant-optimized")]
use blake2b_custom::Blake2bContext;

#[cfg(feature = "variant-simd")]
mod blake2b_simd;
#[cfg(feature = "variant-simd")]
use blake2b_simd::Blake2bSimdContext;

const CHALLENGE_SIZE_BYTES: usize = 128;

/// Solver for Friendly Captcha Proof-of-Work puzzles
///
/// This implementation uses Blake2b-256 hashing to find a nonce that produces
/// a hash value below a given threshold. It's compatible with the Friendly Captcha
/// puzzle format.
#[wasm_bindgen]
pub struct Solver {
    buffer: Box<[u8; CHALLENGE_SIZE_BYTES]>,
}

#[wasm_bindgen]
impl Solver {
    /// Create a new solver instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            buffer: Box::new([0u8; CHALLENGE_SIZE_BYTES]),
        }
    }

    /// Solve the Blake2b hashing problem
    ///
    /// This only changes the last 4 bytes of the input array to find a solution.
    ///
    /// # Arguments
    /// * `puzzle` - The puzzle buffer (32-64 bytes, will be padded to 128)
    /// * `threshold` - u32 value under which the solution's hash should be
    /// * `max_attempts` - Maximum number of nonces to try (default: u32::MAX)
    ///
    /// # Returns
    /// * Hash as Uint8Array if solution found, empty array otherwise
    #[wasm_bindgen]
    #[cfg(feature = "variant-basic")]
    pub fn solve_blake2b(&mut self, puzzle: &[u8], threshold: u32, max_attempts: u32) -> Vec<u8> {
        // Validate input
        if puzzle.is_empty() || puzzle.len() > CHALLENGE_SIZE_BYTES {
            return Vec::new();
        }

        // Copy puzzle into working buffer and pad with zeros
        self.buffer[..puzzle.len()].copy_from_slice(puzzle);
        if puzzle.len() < CHALLENGE_SIZE_BYTES {
            self.buffer[puzzle.len()..].fill(0);
        }

        // Read starting nonce from last 4 bytes (little-endian)
        let start = u32::from_le_bytes([
            self.buffer[124],
            self.buffer[125],
            self.buffer[126],
            self.buffer[127],
        ]);

        // Calculate end point (with wrapping to handle overflow)
        let end = start.wrapping_add(max_attempts);

        // Try each nonce
        for nonce in start..end {
            // Update last 4 bytes with current nonce (little-endian)
            self.buffer[124..128].copy_from_slice(&nonce.to_le_bytes());

            // Hash the buffer using Blake2b-256
            let hash = Blake2b::<U32>::new()
                .chain_update(&self.buffer[..])
                .finalize();

            // Check first 4 bytes of hash as little-endian u32
            let hash_value = u32::from_le_bytes([
                hash[0],
                hash[1],
                hash[2],
                hash[3],
            ]);

            // If hash is below threshold, we found a solution!
            if hash_value < threshold {
                return hash.to_vec();
            }
        }

        // No solution found
        Vec::new()
    }

    /// Get the current buffer contents (includes the solution nonce in last 4 bytes)
    #[wasm_bindgen]
    pub fn get_solution(&self) -> Vec<u8> {
        self.buffer.to_vec()
    }

    /// Get just the solution nonce (last 8 bytes of buffer)
    #[wasm_bindgen]
    pub fn get_solution_nonce(&self) -> Vec<u8> {
        self.buffer[120..128].to_vec()
    }

    /// Solve using optimized Blake2b with state reuse (50% faster)
    ///
    /// This variant implements the AssemblyScript optimization: instead of creating
    /// a new hasher for each iteration, we reset only the hash state vector between
    /// attempts. This is the key optimization that makes WASM ~50% faster.
    ///
    /// # Arguments
    /// * `puzzle` - The puzzle buffer (32-64 bytes, will be padded to 128)
    /// * `threshold` - u32 value under which the solution's hash should be
    /// * `max_attempts` - Maximum number of nonces to try
    ///
    /// # Returns
    /// * Hash as Uint8Array if solution found, empty array otherwise
    #[wasm_bindgen]
    #[cfg(feature = "variant-optimized")]
    pub fn solve_blake2b_optimized(&mut self, puzzle: &[u8], threshold: u32, max_attempts: u32) -> Vec<u8> {
        // Validate input
        if puzzle.is_empty() || puzzle.len() > CHALLENGE_SIZE_BYTES {
            return Vec::new();
        }

        // Copy puzzle into working buffer and pad with zeros
        self.buffer[..puzzle.len()].copy_from_slice(puzzle);
        if puzzle.len() < CHALLENGE_SIZE_BYTES {
            self.buffer[puzzle.len()..].fill(0);
        }

        // Create Blake2b context once (outside the loop)
        let mut ctx = Blake2bContext::new(32);
        ctx.t = CHALLENGE_SIZE_BYTES as u64;

        // Read starting nonce from last 4 bytes (little-endian)
        let start = u32::from_le_bytes([
            self.buffer[124],
            self.buffer[125],
            self.buffer[126],
            self.buffer[127],
        ]);

        // Calculate end point (with wrapping to handle overflow)
        let end = start.wrapping_add(max_attempts);

        // Try each nonce
        for nonce in start..end {
            // Update last 4 bytes with current nonce (little-endian)
            self.buffer[124..128].copy_from_slice(&nonce.to_le_bytes());

            // Reset only the hash state (NOT the buffer) - this is the optimization!
            ctx.reset_for_short_message();

            // Compress the 128-byte buffer
            ctx.compress(&self.buffer[..], true);

            // Check first 4 bytes of hash as little-endian u32
            let hash_value = ctx.hash_value_u32();

            // If hash is below threshold, we found a solution!
            if hash_value < threshold {
                return ctx.finalize();
            }
        }

        // No solution found
        Vec::new()
    }

    /// Solve using SIMD-optimized Blake2b (requires wasm32-simd128 build)
    ///
    /// This variant uses WASM SIMD intrinsics (v128, i64x2_*) for vectorized
    /// operations in Blake2b compression. Requires browsers with WASM SIMD support.
    ///
    /// # Arguments
    /// * `puzzle` - The puzzle buffer (32-64 bytes, will be padded to 128)
    /// * `threshold` - u32 value under which the solution's hash should be
    /// * `max_attempts` - Maximum number of nonces to try
    ///
    /// # Returns
    /// * Hash as Uint8Array if solution found, empty array otherwise
    #[wasm_bindgen]
    #[cfg(feature = "variant-simd")]
    pub fn solve_blake2b_simd(&mut self, puzzle: &[u8], threshold: u32, max_attempts: u32) -> Vec<u8> {
        // Validate input
        if puzzle.is_empty() || puzzle.len() > CHALLENGE_SIZE_BYTES {
            return Vec::new();
        }

        // Copy puzzle into working buffer and pad with zeros
        self.buffer[..puzzle.len()].copy_from_slice(puzzle);
        if puzzle.len() < CHALLENGE_SIZE_BYTES {
            self.buffer[puzzle.len()..].fill(0);
        }

        // Create SIMD Blake2b context once (outside the loop)
        let mut ctx = Blake2bSimdContext::new(32);
        ctx.t = CHALLENGE_SIZE_BYTES as u64;

        // Read starting nonce from last 4 bytes (little-endian)
        let start = u32::from_le_bytes([
            self.buffer[124],
            self.buffer[125],
            self.buffer[126],
            self.buffer[127],
        ]);

        // Calculate end point (with wrapping to handle overflow)
        let end = start.wrapping_add(max_attempts);

        // Try each nonce
        for nonce in start..end {
            // Update last 4 bytes with current nonce (little-endian)
            self.buffer[124..128].copy_from_slice(&nonce.to_le_bytes());

            // Reset only the hash state (NOT the buffer) - optimization!
            ctx.reset_for_short_message();

            // Compress the 128-byte buffer using SIMD operations
            ctx.compress(&self.buffer[..], true);

            // Check first 4 bytes of hash as little-endian u32
            let hash_value = ctx.hash_value_u32();

            // If hash is below threshold, we found a solution!
            if hash_value < threshold {
                return ctx.finalize();
            }
        }

        // No solution found
        Vec::new()
    }
}

/// Calculate the difficulty threshold from a difficulty byte
///
/// Formula: T = floor(2^((255.999-d)/8))
///
/// # Arguments
/// * `difficulty` - Difficulty byte (0-255)
///
/// # Returns
/// * Threshold value as u32
#[wasm_bindgen]
pub fn difficulty_to_threshold(difficulty: u8) -> u32 {
    let exponent = (255.999 - f64::from(difficulty)) / 8.0;
    2_f64.powf(exponent).floor() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(all(feature = "variant-basic", feature = "variant-optimized"))]
    fn test_custom_blake2b_vs_standard() {
        // Verify custom Blake2b produces same results as standard crate
        let data = [0x42u8; 128];

        // Standard crate
        use blake2::digest::Digest;
        let standard_hash = Blake2b::<U32>::new()
            .chain_update(&data)
            .finalize();

        // Custom implementation
        let mut ctx = Blake2bContext::new(32);
        ctx.t = 128;
        ctx.compress(&data, true);
        let custom_hash = ctx.finalize();

        assert_eq!(
            standard_hash.as_slice(),
            custom_hash.as_slice(),
            "Custom Blake2b should match standard implementation"
        );
    }

    #[test]
    #[cfg(all(feature = "variant-basic", feature = "variant-optimized"))]
    fn test_optimized_vs_basic() {
        // Verify optimized variant produces same results as basic
        let mut solver = Solver::new();
        let puzzle = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let threshold = difficulty_to_threshold(50);

        let hash_basic = solver.solve_blake2b(&puzzle, threshold, 1_000_000);
        let hash_optimized = solver.solve_blake2b_optimized(&puzzle, threshold, 1_000_000);

        assert_eq!(hash_basic, hash_optimized, "Optimized should match basic");
    }

    #[test]
    #[cfg(all(feature = "variant-optimized", feature = "variant-simd"))]
    fn test_simd_vs_optimized() {
        // Verify SIMD variant produces same results as optimized
        let mut solver = Solver::new();
        let puzzle = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let threshold = difficulty_to_threshold(50);

        let hash_optimized = solver.solve_blake2b_optimized(&puzzle, threshold, 1_000_000);
        let hash_simd = solver.solve_blake2b_simd(&puzzle, threshold, 1_000_000);

        assert_eq!(hash_optimized, hash_simd, "SIMD should match optimized");
    }

    #[test]
    #[cfg(feature = "variant-simd")]
    fn test_simd_blake2b_vs_standard() {
        use blake2b_custom::Blake2bContext;
        use blake2b_simd::Blake2bSimdContext;

        // Verify SIMD Blake2b produces same results as custom Blake2b
        let data = [0x42u8; 128];

        // Custom implementation
        let mut ctx_custom = Blake2bContext::new(32);
        ctx_custom.t = 128;
        ctx_custom.compress(&data, true);
        let custom_hash = ctx_custom.finalize();

        // SIMD implementation
        let mut ctx_simd = Blake2bSimdContext::new(32);
        ctx_simd.t = 128;
        ctx_simd.compress(&data, true);
        let simd_hash = ctx_simd.finalize();

        assert_eq!(
            custom_hash.as_slice(),
            simd_hash.as_slice(),
            "SIMD Blake2b should match custom Blake2b"
        );
    }

    #[test]
    fn test_difficulty_to_threshold() {
        // Test easy difficulty (d=0) should give max threshold
        let t0 = difficulty_to_threshold(0);
        assert!(t0 > 4_000_000_000);

        // Test medium difficulty
        let t120 = difficulty_to_threshold(120);
        assert!(t120 > 100_000 && t120 < 200_000);

        // Test hard difficulty
        let t200 = difficulty_to_threshold(200);
        assert!(t200 > 100 && t200 < 200);

        // Test extreme difficulty
        let t255 = difficulty_to_threshold(255);
        assert_eq!(t255, 1);
    }

    #[test]
    #[cfg(feature = "variant-basic")]
    fn test_solver_easy_puzzle() {
        let mut solver = Solver::new();

        // Create a simple puzzle
        let mut puzzle = vec![0u8; 128];
        puzzle[0] = 1;
        puzzle[1] = 2;
        puzzle[2] = 3;

        // Very easy difficulty - should find solution quickly
        let threshold = difficulty_to_threshold(50);
        let hash = solver.solve_blake2b(&puzzle, threshold, 1_000_000);

        assert!(!hash.is_empty(), "Should find a solution for easy puzzle");
        assert_eq!(hash.len(), 32, "Hash should be 32 bytes");
    }

    #[test]
    #[cfg(feature = "variant-basic")]
    fn test_solver_no_solution() {
        let mut solver = Solver::new();

        let puzzle = vec![0u8; 128];

        // Impossible threshold with very few attempts
        let hash = solver.solve_blake2b(&puzzle, 1, 100);

        assert!(hash.is_empty(), "Should return empty when no solution found");
    }

    #[test]
    #[cfg(feature = "variant-basic")]
    fn test_solution_nonce() {
        let mut solver = Solver::new();

        let puzzle = vec![0u8; 128];
        let threshold = difficulty_to_threshold(50);

        solver.solve_blake2b(&puzzle, threshold, 1_000_000);

        let solution = solver.get_solution();
        assert_eq!(solution.len(), 128);

        let nonce = solver.get_solution_nonce();
        assert_eq!(nonce.len(), 8);
    }
}

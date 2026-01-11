// Custom Blake2b implementation optimized for Friendly POW
// Based on the AssemblyScript blake2b64.ts from friendly-pow
//
// This implementation exposes internal state to allow the optimization where
// we reset only the hash state between iterations, not the entire buffer.

const BLAKE2B_IV: [u64; 8] = [
    0x6a09e667f3bcc908,
    0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b,
    0xa54ff53a5f1d36f1,
    0x510e527fade682d1,
    0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b,
    0x5be0cd19137e2179,
];

// SIGMA permutation table for Blake2b (12 rounds)
const SIGMA: [[usize; 16]; 12] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
];

#[inline(always)]
fn rotr64(x: u64, n: u32) -> u64 {
    (x >> n) | (x << (64 - n))
}

/// Blake2b context for friendly-pow optimization
pub struct Blake2bContext {
    /// Hash state (8 × u64)
    pub h: [u64; 8],
    /// Input byte count
    pub t: u64,
    /// Output length
    pub outlen: usize,
}

impl Blake2bContext {
    /// Create a new Blake2b context
    pub fn new(outlen: usize) -> Self {
        assert!(outlen > 0 && outlen <= 64, "Invalid output length");

        let mut h = BLAKE2B_IV;
        // Mix output length into h[0]
        h[0] ^= 0x01010000 ^ (outlen as u64);

        Self { h, t: 0, outlen }
    }

    /// Reset only the hash state to IV (optimization for fixed 128-byte input)
    ///
    /// This is the key optimization from the AssemblyScript version.
    /// We reset only h[] to IV, NOT the message buffer.
    #[inline(always)]
    pub fn reset_for_short_message(&mut self) {
        self.h = BLAKE2B_IV;
        self.h[0] ^= 0x01010000 ^ (self.outlen as u64);
    }

    /// Compress a 128-byte block
    ///
    /// This is called with the full 128-byte buffer that contains the nonce
    /// in its last 4 bytes.
    pub fn compress(&mut self, buffer: &[u8], is_last: bool) {
        assert_eq!(buffer.len(), 128, "Buffer must be exactly 128 bytes");

        // Load message block as 16 × u64 (little-endian)
        let mut m = [0u64; 16];
        for i in 0..16 {
            m[i] = u64::from_le_bytes([
                buffer[i * 8],
                buffer[i * 8 + 1],
                buffer[i * 8 + 2],
                buffer[i * 8 + 3],
                buffer[i * 8 + 4],
                buffer[i * 8 + 5],
                buffer[i * 8 + 6],
                buffer[i * 8 + 7],
            ]);
        }

        // Initialize working vector
        let mut v = [0u64; 16];
        v[0..8].copy_from_slice(&self.h);
        v[8..12].copy_from_slice(&BLAKE2B_IV[0..4]);
        v[12] = BLAKE2B_IV[4] ^ self.t; // Mix in byte counter
        v[13] = BLAKE2B_IV[5];
        v[14] = if is_last {
            !BLAKE2B_IV[6]
        } else {
            BLAKE2B_IV[6]
        };
        v[15] = BLAKE2B_IV[7];

        // 12 rounds of mixing
        for round in 0..12 {
            let s = &SIGMA[round];

            // Mix columns
            Self::g(&mut v, 0, 4, 8, 12, m[s[0]], m[s[1]]);
            Self::g(&mut v, 1, 5, 9, 13, m[s[2]], m[s[3]]);
            Self::g(&mut v, 2, 6, 10, 14, m[s[4]], m[s[5]]);
            Self::g(&mut v, 3, 7, 11, 15, m[s[6]], m[s[7]]);

            // Mix diagonals
            Self::g(&mut v, 0, 5, 10, 15, m[s[8]], m[s[9]]);
            Self::g(&mut v, 1, 6, 11, 12, m[s[10]], m[s[11]]);
            Self::g(&mut v, 2, 7, 8, 13, m[s[12]], m[s[13]]);
            Self::g(&mut v, 3, 4, 9, 14, m[s[14]], m[s[15]]);
        }

        // Mix the upper and lower halves of v into h
        for i in 0..8 {
            self.h[i] ^= v[i] ^ v[i + 8];
        }
    }

    /// Blake2b G mixing function
    #[inline(always)]
    fn g(v: &mut [u64; 16], a: usize, b: usize, c: usize, d: usize, x: u64, y: u64) {
        v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
        v[d] = rotr64(v[d] ^ v[a], 32);
        v[c] = v[c].wrapping_add(v[d]);
        v[b] = rotr64(v[b] ^ v[c], 24);
        v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
        v[d] = rotr64(v[d] ^ v[a], 16);
        v[c] = v[c].wrapping_add(v[d]);
        v[b] = rotr64(v[b] ^ v[c], 63);
    }

    /// Finalize and return hash as bytes
    pub fn finalize(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.outlen);
        for i in 0..(self.outlen / 8) {
            out.extend_from_slice(&self.h[i].to_le_bytes());
        }
        // Handle partial last u64 if outlen is not multiple of 8
        if self.outlen % 8 != 0 {
            let remaining = self.outlen % 8;
            out.extend_from_slice(&self.h[self.outlen / 8].to_le_bytes()[..remaining]);
        }
        out
    }

    /// Get the first 4 bytes of the hash as u32 (little-endian)
    ///
    /// This is used for the threshold check in the solver
    #[inline(always)]
    pub fn hash_value_u32(&self) -> u32 {
        (self.h[0] & 0xFFFFFFFF) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake2b_empty() {
        let mut ctx = Blake2bContext::new(32);
        ctx.t = 0;

        // Empty message hash (not really valid but testing basics)
        let buffer = [0u8; 128];
        ctx.compress(&buffer, true);
        let hash = ctx.finalize();

        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_blake2b_reset() {
        let mut ctx = Blake2bContext::new(32);
        ctx.t = 128;

        let buffer = [0x42u8; 128];
        ctx.compress(&buffer, true);
        let hash1 = ctx.finalize();

        // Reset and hash again
        ctx.reset_for_short_message();
        ctx.compress(&buffer, true);
        let hash2 = ctx.finalize();

        assert_eq!(hash1, hash2, "Reset should produce same hash for same input");
    }

    #[test]
    fn test_hash_value_u32() {
        let mut ctx = Blake2bContext::new(32);
        ctx.t = 128;

        let buffer = [0u8; 128];
        ctx.compress(&buffer, true);

        let val = ctx.hash_value_u32();
        let hash = ctx.finalize();

        let expected = u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
        assert_eq!(val, expected);
    }
}

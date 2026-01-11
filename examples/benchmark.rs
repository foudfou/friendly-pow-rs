use friendly_pow_rs::{Solver, difficulty_to_threshold};
use std::time::Instant;

fn main() {
    println!("🦀 Friendly POW Rust Benchmark - All Variants\n");

    let difficulties = vec![50, 80, 120];
    let mut variants = Vec::new();

    #[cfg(feature = "variant-basic")]
    variants.push(("Basic", SolverVariant::Basic));

    #[cfg(feature = "variant-optimized")]
    variants.push(("Optimized", SolverVariant::Optimized));

    #[cfg(feature = "variant-simd")]
    variants.push(("SIMD", SolverVariant::Simd));

    for difficulty in &difficulties {
        let threshold = difficulty_to_threshold(*difficulty);
        let expected_attempts = (u32::MAX as f64 / threshold as f64) as u64;

        println!("═══════════════════════════════════════════════");
        println!("Difficulty: {}", difficulty);
        println!("Threshold: {}", threshold);
        println!("Expected attempts: ~{}", expected_attempts);
        println!("═══════════════════════════════════════════════\n");

        let mut results = Vec::new();

        for (name, variant) in &variants {
            let mut solver = Solver::new();

            // Create a simple puzzle
            let mut puzzle = vec![0u8; 32];
            puzzle[0] = 1;
            puzzle[1] = 2;
            puzzle[2] = 3;
            puzzle[3] = *difficulty;

            let start = Instant::now();
            let hash = match variant {
                #[cfg(feature = "variant-basic")]
                SolverVariant::Basic => solver.solve_blake2b(&puzzle, threshold, 100_000_000),
                #[cfg(feature = "variant-optimized")]
                SolverVariant::Optimized => solver.solve_blake2b_optimized(&puzzle, threshold, 100_000_000),
                #[cfg(feature = "variant-simd")]
                SolverVariant::Simd => solver.solve_blake2b_simd(&puzzle, threshold, 100_000_000),
                #[cfg(not(any(feature = "variant-basic", feature = "variant-optimized", feature = "variant-simd")))]
                _ => Vec::new(),
            };
            let duration = start.elapsed();

            if !hash.is_empty() {
                let time_secs = duration.as_secs_f64();
                let hash_rate = expected_attempts as f64 / time_secs / 1_000_000.0;

                println!("  {} Variant:", name);
                println!("    ✅ Solution found!");
                println!("    Time: {:.3}s", time_secs);
                println!("    Hash rate: {:.2} MH/s", hash_rate);
                println!("    Hash: {}", hex::encode(&hash[..8]));
                println!();

                results.push((*name, hash_rate, hash));
            } else {
                println!("  {} Variant:", name);
                println!("    ❌ No solution found (increase max attempts)");
                println!();
            }
        }

        // Verify all variants produce the same hash
        if results.len() == variants.len() {
            let reference_hash = &results[0].2;
            let all_match = results.iter().all(|(_, _, h)| h == reference_hash);

            if all_match {
                println!("  ✅ All variants produced identical hashes");
            } else {
                println!("  ⚠️  WARNING: Variants produced different hashes!");
            }

            // Compare performance
            let basic_rate = results[0].1;
            println!("\n  Performance vs Basic:");
            for (name, rate, _) in &results[1..] {
                let speedup = (rate / basic_rate - 1.0) * 100.0;
                println!("    {} : {:+.1}% ({:.2} MH/s)", name, speedup, rate);
            }
        }

        println!("\n");
    }
}

enum SolverVariant {
    Basic,
    Optimized,
    Simd,
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

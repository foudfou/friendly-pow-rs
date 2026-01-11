use friendly_pow_rs::{Solver, difficulty_to_threshold};
use std::time::Instant;

fn main() {
    println!("🦀 Friendly POW Rust Benchmark\n");

    let mut solver = Solver::new();

    let difficulties = vec![50, 80, 120, 150];

    for difficulty in difficulties {
        let threshold = difficulty_to_threshold(difficulty);
        let expected_attempts = (u32::MAX as f64 / threshold as f64) as u64;

        println!("Difficulty: {}", difficulty);
        println!("Threshold: {}", threshold);
        println!("Expected attempts: ~{}", expected_attempts);

        // Create a simple puzzle
        let mut puzzle = vec![0u8; 32];
        puzzle[0] = 1;
        puzzle[1] = 2;
        puzzle[2] = 3;
        puzzle[3] = difficulty;

        let start = Instant::now();
        let hash = solver.solve_blake2b(&puzzle, threshold, 100_000_000);
        let duration = start.elapsed();

        if !hash.is_empty() {
            let time_secs = duration.as_secs_f64();
            let hash_rate = (expected_attempts as f64 / time_secs / 1_000_000.0).round();

            println!("✅ Solution found!");
            println!("Time: {:.3}s", time_secs);
            println!("Estimated hash rate: ~{} MH/s", hash_rate);
            println!("Hash: {}", hex::encode(&hash));
        } else {
            println!("❌ No solution found (increase max attempts)");
        }

        println!();
    }
}

mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

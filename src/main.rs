//! awslc-boot-bench — isolate aws-lc-rs jitter-entropy init cost
//!
//! Measures wall-clock time for:
//!   Phase 0 – Baseline (process start → main entry)
//!   Phase 1 – First SystemRandom::new()  (provider construction)
//!   Phase 2 – First rand::fill()          (triggers CSPRNG seed / jitter entropy)
//!   Phase 3 – Second rand::fill()         (should be fast — pool already seeded)
//!   Phase 4 – 100× rand::fill() loop      (amortised per-call cost)
//!
//! Build:
//!   make release          # or: cargo build --release --target x86_64-unknown-linux-musl
//!
//! Run:
//!   ./target/x86_64-unknown-linux-musl/release/awslc-boot-bench

use std::time::Instant;

/// Record a named phase and return the new Instant.
#[inline(never)]
fn phase(label: &str, start: Instant) -> Instant {
    let elapsed = start.elapsed();
    println!(
        "{:<52} {:>10.3} ms",
        label,
        elapsed.as_secs_f64() * 1_000.0
    );
    Instant::now()
}

fn main() {
    // ── Phase 0: process-start → main() ────────────────────────────
    let t0 = Instant::now();
    println!("awslc-boot-bench v0.1.0");
    println!("{}", "=".repeat(65));

    // Print environment hints
    if let Ok(val) = std::env::var("AWS_LC_FIPS_ENTROPY") {
        println!("AWS_LC_FIPS_ENTROPY = {val}");
    }
    println!(
        "Kernel entropy (if readable): {}",
        std::fs::read_to_string("/proc/sys/kernel/random/entropy_avail")
            .unwrap_or_else(|_| "N/A".into())
            .trim()
    );
    println!("{}", "-".repeat(65));

    let t1 = phase("Phase 0  main() entry + env probe", t0);

    // ── Phase 1: Construct the SystemRandom provider ───────────────
    let rng = aws_lc_rs::rand::SystemRandom::new();
    let t2 = phase("Phase 1  SystemRandom::new()", t1);

    // ── Phase 2: First fill — triggers jitter-entropy seeding ──────
    let mut buf = [0u8; 32];
    aws_lc_rs::rand::SecureRandom::fill(&rng, &mut buf)
        .expect("first fill failed");
    let t3 = phase("Phase 2  First  rand::fill(32 B)  ← SEED COST", t2);
    println!("         first 8 bytes: {:02x?}", &buf[..8]);

    // ── Phase 3: Second fill — pool already seeded ─────────────────
    let mut buf2 = [0u8; 32];
    aws_lc_rs::rand::SecureRandom::fill(&rng, &mut buf2)
        .expect("second fill failed");
    let t4 = phase("Phase 3  Second rand::fill(32 B)", t3);

    // ── Phase 4: 100× fill — amortised cost ────────────────────────
    let iterations: u32 = 100;
    let mut scratch = [0u8; 32];
    for _ in 0..iterations {
        aws_lc_rs::rand::SecureRandom::fill(&rng, &mut scratch)
            .expect("loop fill failed");
    }
    let t5 = phase(
        &format!("Phase 4  {iterations}× rand::fill(32 B) total"),
        t4,
    );

    let per_call_ns = t4.elapsed().as_nanos() as f64 / f64::from(iterations);

    println!("{}", "-".repeat(65));
    println!(
        "{:<52} {:>10.3} ms",
        "Total wall-clock (Phases 0–4)",
        t0.elapsed().as_secs_f64() * 1_000.0
    );
    println!(
        "{:<52} {:>10.0} ns",
        "Amortised per-call (Phase 4)",
        per_call_ns
    );
    println!("{}", "=".repeat(65));

    // ── Suppress optimiser elision ─────────────────────────────────
    std::hint::black_box(&buf);
    std::hint::black_box(&buf2);
    std::hint::black_box(&scratch);
    let _ = t5;
}

#![forbid(unsafe_code)]

//! Manual capacity benchmark — integration layer (Phase 3).
//!
//! Production capacity evidence for the coordination, reconciliation, and
//! switching layers. These tests are `#[ignore]`d because they are measuring
//! instruments, not correctness gates; run them explicitly:
//!
//! ```powershell
//! cargo test -p wellfit_integration --release --test capacity -- --ignored --nocapture
//! ```

use std::time::Instant;
use wellfit_core::audit_integrity::{compute_hash, verify_hash_chain};
use wellfit_integration::{CoordinationEngine, CoordinationMode, SwitchingDrill};

/// Reconciliation throughput: record + compare 10,000 matching records.
#[test]
#[ignore = "manual capacity benchmark; run with --ignored --nocapture"]
fn reconciliation_throughput_10k() {
    const N: usize = 10_000;

    let mut engine = CoordinationEngine::new();
    for i in 0..N {
        let key = format!("KEY-{i:06}");
        let value = format!("value-{i}");
        engine.record_new(&key, &value);
        engine.record_legacy(&key, &value);
    }

    let start = Instant::now();
    let result = engine.reconcile("capacity-10k");
    let elapsed = start.elapsed();
    let per_sec = N as f64 / elapsed.as_secs_f64();

    println!("[capacity] reconcile {N} records: {elapsed:?} ({per_sec:.0} records/sec)");
    assert_eq!(result.total_records, N);
    assert_eq!(result.matched_records, N);
    assert_eq!(result.diff_records, 0);
}

/// Audit hash chain verification throughput: 10,000 SHA-256 entries.
#[test]
#[ignore = "manual capacity benchmark; run with --ignored --nocapture"]
fn hash_chain_verify_10k() {
    const N: usize = 10_000;

    let hashes: Vec<String> = (0..N)
        .map(|i| compute_hash(&format!("entry-{i}")))
        .collect();

    let start = Instant::now();
    let result = verify_hash_chain(&hashes);
    let elapsed = start.elapsed();
    let per_sec = N as f64 / elapsed.as_secs_f64();

    println!("[capacity] hash chain verify {N}: {elapsed:?} ({per_sec:.0} entries/sec)");
    assert!(result.is_valid());
}

/// End-to-end switching drill: 1,000 records through the full 4-phase lifecycle.
#[test]
#[ignore = "manual capacity benchmark; run with --ignored --nocapture"]
fn switching_drill_full_lifecycle() {
    const N: usize = 1_000;

    let records: Vec<(String, String, String)> = (0..N)
        .map(|i| (format!("K{i:05}"), format!("v{i}"), format!("v{i}")))
        .collect();

    let mut drill = SwitchingDrill::new();
    let start = Instant::now();
    let report = drill.run_drill("capacity-drill", &records);
    let elapsed = start.elapsed();

    println!("[capacity] full switching drill ({N} records, 4 phases): {elapsed:?}");
    let report = report.expect("drill must pass for matching records");
    assert!(report.overall_pass);
    assert_eq!(report.final_mode, CoordinationMode::Decommissioned);
    assert_eq!(report.steps.len(), 4);
}

/// Hash computation throughput: 10,000 SHA-256 computations.
#[test]
#[ignore = "manual capacity benchmark; run with --ignored --nocapture"]
fn hash_compute_10k() {
    const N: usize = 10_000;

    let start = Instant::now();
    let mut last = String::new();
    for i in 0..N {
        last = compute_hash(&format!("payload-{i}"));
    }
    let elapsed = start.elapsed();
    let per_sec = N as f64 / elapsed.as_secs_f64();

    println!("[capacity] sha256 compute {N}: {elapsed:?} ({per_sec:.0} hashes/sec)");
    assert_eq!(last.len(), 64);
}

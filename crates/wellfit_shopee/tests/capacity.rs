#![forbid(unsafe_code)]

//! Manual capacity benchmark — procurement pipeline (Phase 2, W5).
//!
//! Production capacity evidence for the Shopee procurement automation
//! orchestrator. Run explicitly:
//!
//! ```powershell
//! cargo test -p wellfit_shopee --release --test capacity -- --ignored --nocapture
//! ```

use std::sync::{Arc, Mutex};
use std::time::Instant;
use wellfit_core::ledger::IdempotencyLedger;
use wellfit_core::release_gate::ReleaseGate;
use wellfit_shopee::ProcurementAutomation;

/// Generate a synthetic Shopee order export with `rows` lines.
///
/// Uses the two authoritative demo SKUs so rows resolve through the demo MDM
/// catalog; this is synthetic capacity data, never presented as real data.
fn synthetic_export(rows: usize) -> String {
    let mut csv = String::from(
        "ORDER_NO,CUSTOMER_REF,ORDER_DATETIME,ORDER_TOTAL,ITEM_NO,QTY,UNIT_PRICE,LINE_AMOUNT\n",
    );
    let skus = ["HAV-TOP-BLK-39", "HAV-TOP-NVY-41"];
    for i in 0..rows {
        let sku = skus[i % skus.len()];
        csv.push_str(&format!(
            "26{:08},CUST-{},2026-11-04 08:00:00,1200,{},2,600,1200\n",
            i,
            i % 100,
            sku
        ));
    }
    csv
}

/// Procurement pipeline throughput: 2,000 rows in a single batch.
#[test]
#[ignore = "manual capacity benchmark; run with --ignored --nocapture"]
fn procurement_pipeline_2k_rows() {
    const ROWS: usize = 2_000;

    let ledger = Arc::new(IdempotencyLedger::new(":memory:").expect("in-memory ledger"));
    let release_gate = Arc::new(Mutex::new(ReleaseGate::new(vec![])));
    let mut automation = ProcurementAutomation::new(ledger, release_gate);

    let csv = synthetic_export(ROWS);

    let start = Instant::now();
    let result = automation.run_pipeline(&csv, "capacity-2k");
    let elapsed = start.elapsed();
    let per_sec = ROWS as f64 / elapsed.as_secs_f64();

    let result = result.expect("pipeline must complete");
    println!(
        "[capacity] procurement pipeline {ROWS} rows: {elapsed:?} ({per_sec:.0} rows/sec), \
         accepted={}, isolated={}",
        result.orders.len(),
        result.errors.len()
    );

    // The batch must be fully accounted for: every row is either accepted
    // (grouped into an order) or isolated (quarantined with a rule code).
    assert!(result.orders.len() + result.errors.len() >= ROWS);
    assert!(!result.audit_hash.is_empty());
}

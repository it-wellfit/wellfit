#![forbid(unsafe_code)]

//! Cross-module integration tests: wellfit_assist capture + wellfit_core
//! audit integrity + automation framework working together.
//!
//! Proves the Phase 1 (assistive layer) and Phase 2/3 (automation +
//! integration) modules compose: a shadow capture produces evidence whose
//! digest survives the audit-integrity hash chain, and the automation
//! framework tracks the workflow status consistently.

use std::fs;
use std::path::PathBuf;
use wellfit_assist::capture::capture_reports_export;
use wellfit_core::audit_integrity::verify_hash_chain;
use wellfit_core::automation::{CircuitBreaker, WorkflowStatus};

fn temp_root(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("wellfit-xmod-integ-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn write_input(root: &std::path::Path, csv: &str) -> PathBuf {
    let path = root.join("vendor-export.csv");
    fs::write(&path, csv).expect("write input");
    path
}

/// Shadow capture evidence -> audit hash chain verification.
#[test]
fn shadow_capture_evidence_survives_audit_hash_chain() {
    let root = temp_root("hash-chain");
    let artifacts = root.join("artifacts");

    let balanced = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\n8,Top Black,WH-A,500,200,50,0,650\n";
    let mismatched = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\n8,Top Black,WH-A,500,200,50,0,999\n";

    let input1 = write_input(&root, balanced);
    let input2 = root.join("vendor-export-2.csv");
    fs::write(&input2, mismatched).expect("write input 2");

    let capture1 =
        capture_reports_export(&input1, &artifacts, "XMOD-MATCH").expect("capture 1 must succeed");
    let capture2 =
        capture_reports_export(&input2, &artifacts, "XMOD-DIFF").expect("capture 2 must succeed");

    assert!(
        capture1.evidence.row_level_match,
        "balanced input must match"
    );
    assert!(
        !capture2.evidence.row_level_match,
        "mismatched input must flag"
    );

    let hashes: Vec<String> = [
        capture1.evidence.vendor_digest.clone(),
        capture2.evidence.vendor_digest.clone(),
    ]
    .to_vec();
    let integrity = verify_hash_chain(&hashes);
    assert!(
        integrity.is_valid(),
        "audit chain must be structurally valid"
    );

    let _ = fs::remove_dir_all(&root);
}

/// Automation framework + circuit breaker compose with capture outcomes.
#[test]
fn capture_outcome_maps_to_workflow_status_and_breaker() {
    let root = temp_root("workflow");
    let artifacts = root.join("artifacts");

    let mut breaker = CircuitBreaker::default_breaker();

    let balanced = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\n8,Top Black,WH-A,500,200,50,0,650\n";
    let input = write_input(&root, balanced);
    let outcome = capture_reports_export(&input, &artifacts, "WF-OK").expect("capture");
    assert_eq!(outcome.evidence.records_mismatched, 0);

    let mismatched = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\n8,Top Black,WH-A,500,200,50,0,777\n";
    let input2 = root.join("vendor-export-2.csv");
    fs::write(&input2, mismatched).expect("write input 2");
    let outcome2 = capture_reports_export(&input2, &artifacts, "WF-FAIL").expect("capture");
    assert_eq!(outcome2.evidence.records_mismatched, 1);

    let ok_status = if outcome.evidence.row_level_match {
        WorkflowStatus::Success
    } else {
        WorkflowStatus::Failed
    };
    let fail_status = if outcome2.evidence.row_level_match {
        WorkflowStatus::Success
    } else {
        WorkflowStatus::Failed
    };
    assert_eq!(ok_status, WorkflowStatus::Success);
    assert_eq!(fail_status, WorkflowStatus::Failed);

    breaker.record_failure();
    assert!(
        breaker.state == wellfit_core::automation::CircuitState::Closed,
        "1 failure < threshold 5: breaker stays closed"
    );
    breaker.record_success();
    assert_eq!(breaker.consecutive_failures, 0, "success resets counter");

    let _ = fs::remove_dir_all(&root);
}

/// Evidence JSON round-trip: the saved evidence file parses back into the
/// same digest values, proving the on-disk artifact is self-consistent.
#[test]
fn evidence_json_round_trip_preserves_digests() {
    let root = temp_root("json-roundtrip");
    let artifacts = root.join("artifacts");

    let csv = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\n8,Top Black,WH-A,500,200,50,0,650\n";
    let input = write_input(&root, csv);
    let outcome = capture_reports_export(&input, &artifacts, "JSON-RT").expect("capture");

    let saved = fs::read_to_string(&outcome.evidence_path).expect("read evidence");
    let parsed: serde_json::Value = serde_json::from_str(&saved).expect("evidence is valid JSON");

    assert_eq!(
        parsed["schema"].as_str().unwrap_or(""),
        "wellfit-shadow-evidence/v1"
    );
    assert_eq!(
        parsed["vendor_digest"].as_str().unwrap_or(""),
        outcome.evidence.vendor_digest,
        "vendor digest survives the JSON round trip"
    );
    assert_eq!(
        parsed["our_digest"].as_str().unwrap_or(""),
        outcome.evidence.our_digest,
        "our digest survives the JSON round trip"
    );
    assert_eq!(
        parsed["records_verified"].as_u64().unwrap_or(0),
        outcome.evidence.records_verified as u64
    );

    let _ = fs::remove_dir_all(&root);
}

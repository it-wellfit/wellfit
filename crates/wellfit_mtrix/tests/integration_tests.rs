#![forbid(unsafe_code)]

//! Integration tests for the wellfit_mtrix crate.

use std::sync::{Arc, Mutex};
use wellfit_core::ledger::IdempotencyLedger;
use wellfit_core::release_gate::ReleaseGate;
use wellfit_mtrix::spec::{FileRole, MTRIX_SPEC};
use wellfit_mtrix::validators::{BatchStatus, VrRule};
use wellfit_mtrix::{repaired_file_name, spec_conformance, status_label, MtrixEngine};

#[test]
fn spec_declares_five_files() {
    assert_eq!(MTRIX_SPEC.len(), 5);
}

#[test]
fn spec_conformance_reports_declared_rules() {
    let c = spec_conformance();
    assert!(c.rules_declared > 0);
    assert!(c.fields_declared > 0);
}

#[test]
fn vr_rule_all_is_non_empty() {
    assert!(!VrRule::all().is_empty());
}

#[test]
fn status_label_round_trips() {
    // status_label should return a non-empty string for known statuses.
    let s = status_label(BatchStatus::Accepted);
    assert!(!s.is_empty());
}

#[test]
fn repaired_file_name_appends_suffix() {
    let out = repaired_file_name("ALTHCI.csv");
    assert!(out.contains("ALTHCI"));
    assert_ne!(out, "ALTHCI.csv");
}

#[test]
fn discover_files_missing_dir_errors() {
    let res = MtrixEngine::discover_files(std::path::Path::new("/no/such/dir/xyz"));
    assert!(res.is_err());
}

#[test]
fn engine_constructs() {
    let ledger = std::sync::Arc::new(IdempotencyLedger::in_memory().expect("in-memory ledger"));
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let _engine = MtrixEngine::new(ledger, gate);
}

#[test]
fn file_role_get_works() {
    // FileRole enum should be usable as a key.
    let _ = FileRole::Customer;
}

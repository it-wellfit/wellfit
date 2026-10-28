#![forbid(unsafe_code)]

//! Integration tests for the wellfit_reports crate.

use std::sync::{Arc, Mutex};
use wellfit_core::ledger::IdempotencyLedger;
use wellfit_core::release_gate::ReleaseGate;
use wellfit_reports::{InventoryReportRecord, ReportAutomator};

#[test]
fn verify_balance_holds() {
    let rec = InventoryReportRecord {
        item_no: "IT-1".to_string(),
        item_name: "item".to_string(),
        warehouse_code: "W1".to_string(),
        begin_inv: 10,
        in_qty: 5,
        out_qty: 3,
        adj_qty: 1,
        end_inv: 13,
    };
    assert!(rec.verify_balance());
}

#[test]
fn verify_balance_rejects_mismatch() {
    let rec = InventoryReportRecord {
        item_no: "IT-2".to_string(),
        item_name: "item".to_string(),
        warehouse_code: "W1".to_string(),
        begin_inv: 10,
        in_qty: 5,
        out_qty: 3,
        adj_qty: 1,
        end_inv: 99,
    };
    assert!(!rec.verify_balance());
}

#[test]
fn verify_balance_overflow_safe() {
    let rec = InventoryReportRecord {
        item_no: "IT-3".to_string(),
        item_name: "item".to_string(),
        warehouse_code: "W1".to_string(),
        begin_inv: i64::MAX,
        in_qty: 1,
        out_qty: 0,
        adj_qty: 0,
        end_inv: 0,
    };
    assert!(!rec.verify_balance());
}

#[test]
fn automator_constructs() {
    let ledger = std::sync::Arc::new(IdempotencyLedger::in_memory().expect("ledger"));
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let _a = ReportAutomator::new(ledger, gate);
}

#[test]
fn column_contract_non_empty() {
    let c = wellfit_reports::fuyi_inventory_report_contract();
    assert!(!c.required().is_empty());
}

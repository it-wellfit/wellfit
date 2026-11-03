#![forbid(unsafe_code)]

//! Integration tests for the wellfit_shopee crate.

use std::sync::{Arc, Mutex};
use wellfit_core::ledger::IdempotencyLedger;
use wellfit_core::release_gate::ReleaseGate;
use wellfit_shopee::{ShopeeInboundEngine, ShopeeOrder, ShopeeOrderItem};

#[test]
fn order_verify_line_sum_empty() {
    let order = ShopeeOrder {
        order_no: "O1".to_string(),
        customer_ref: "c".to_string(),
        erp_customer_code: "EC".to_string(),
        order_datetime: "2024-01-01".to_string(),
        order_total_cents: 0,
        items: vec![],
    };
    assert!(order.verify_line_sum());
}

#[test]
fn shopee_contract_non_empty() {
    let c = wellfit_shopee::shopee_order_contract();
    assert!(!c.required().is_empty());
}

#[test]
fn engine_constructs() {
    let ledger = std::sync::Arc::new(IdempotencyLedger::in_memory().expect("ledger"));
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let _e = ShopeeInboundEngine::new(ledger, gate);
}

#[test]
fn line_sum_matches() {
    let item = ShopeeOrderItem {
        item_no: "I1".to_string(),
        seller_sku: "S1".to_string(),
        erp_item_no: "E1".to_string(),
        ean13: "123".to_string(),
        qty: 2,
        unit_price_cents: 100,
        line_amount_cents: 200,
    };
    let order = ShopeeOrder {
        order_no: "O2".to_string(),
        customer_ref: "c".to_string(),
        erp_customer_code: "EC".to_string(),
        order_datetime: "2024-01-01".to_string(),
        order_total_cents: 200,
        items: vec![item],
    };
    assert!(order.verify_line_sum());
}

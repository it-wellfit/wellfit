#![forbid(unsafe_code)]

//! Integration tests for the wellfit_bankpay crate.

use std::sync::{Arc, Mutex};
use wellfit_bankpay::{
    base64_decode_media, base64_encode_media, cents_to_yuan_str, media_file_sha256, BankEncoding,
    BankpayEngine, TwdMediaRecord,
};
use wellfit_core::ledger::{BatchScope, IdempotencyLedger};
use wellfit_core::release_gate::ReleaseGate;

fn test_ledger() -> IdempotencyLedger {
    IdempotencyLedger::new_scoped(":memory:", BatchScope::Exclusive).unwrap()
}

fn write_tmp_csv(content: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "bankpay_integration_{}_{}.csv",
        std::process::id(),
        n
    ));
    std::fs::write(&path, content).unwrap();
    path
}

#[test]
fn export_vouchers_success() {
    let ledger = Arc::new(test_ledger());
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let mut engine = BankpayEngine::new(ledger, gate);

    let path = write_tmp_csv(
        "VOUCHER_NO,VENDOR_CODE,AMOUNT,BANK_ACCOUNT,DUE_DATE,REFERENCE\n\
         AP-001,V001,1000.0000,ACCT-1,2026-11-03,INV-100\n\
         AP-002,V002,2500.5000,ACCT-2,2026-11-04,INV-101\n",
    );

    let result = engine.export_vouchers(
        &path,
        "BATCH-INT-1",
        "2026-11-02",
        BankEncoding::Utf8,
        |_| Ok(true),
    );

    assert!(result.is_ok(), "export must succeed: {:?}", result.err());
    let r = result.unwrap();
    assert_eq!(r.vouchers_accepted, 2);
    assert_eq!(r.vouchers_isolated, 0);
    assert!(r.dispatched);
    assert_eq!(r.file_sha256.len(), 64);
    assert!(r.media_file.ends_with(".bankpay.media"));
}

#[test]
fn export_vouchers_isolates_bad_rows() {
    let ledger = Arc::new(test_ledger());
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let mut engine = BankpayEngine::new(ledger, gate);

    let path = write_tmp_csv(
        "VOUCHER_NO,VENDOR_CODE,AMOUNT,BANK_ACCOUNT\nAP-001,V001,notanumber,ACCT-1\n",
    );

    let result = engine.export_vouchers(
        &path,
        "BATCH-INT-2",
        "2026-11-02",
        BankEncoding::Utf8,
        |_| Ok(true),
    );

    assert!(result.is_ok());
    let r = result.unwrap();
    assert_eq!(r.vouchers_accepted, 0);
    assert_eq!(r.vouchers_isolated, 1);
    assert!(!r.dispatched);
}

#[test]
fn import_results_success() {
    let ledger = Arc::new(test_ledger());
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let mut engine = BankpayEngine::new(ledger, gate);

    let path = write_tmp_csv(
        "BANK_REF,VOUCHER_NO,STATUS,AMOUNT,PROCESSED_AT\n\
         EREF-999,AP-001,paid,1000.0000,2026-11-03T09:00:00Z\n",
    );

    let result = engine.import_results(&path, "BATCH-INT-RES", BankEncoding::Utf8);

    assert!(result.is_ok(), "import must succeed: {:?}", result.err());
    let r = result.unwrap();
    assert_eq!(r.results_accepted, 1);
    assert_eq!(r.matched_vouchers, 1);
    assert_eq!(r.unmatched, 0);
}

#[test]
fn query_voucher_flow() {
    use wellfit_bankpay::query;
    let ledger = Arc::new(test_ledger());
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let mut engine = BankpayEngine::new(ledger, gate);

    let result = query(&mut engine, "voucher", "AP-001").unwrap();
    assert_eq!(result.lookup_kind, "voucher");
    assert_eq!(result.voucher_no, Some("AP-001".to_string()));
    assert!(!result.findings.is_empty());
}

#[test]
fn query_tx_flow() {
    use wellfit_bankpay::query;
    let ledger = Arc::new(test_ledger());
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let mut engine = BankpayEngine::new(ledger, gate);

    let result = query(&mut engine, "tx", "EREF-999").unwrap();
    assert_eq!(result.lookup_kind, "tx");
    assert_eq!(result.bank_ref, Some("EREF-999".to_string()));
}

#[test]
fn query_batch_flow() {
    use wellfit_bankpay::query;
    let ledger = Arc::new(test_ledger());
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let mut engine = BankpayEngine::new(ledger, gate);

    let result = query(&mut engine, "batch", "BATCH-X").unwrap();
    assert_eq!(result.lookup_kind, "batch");
    assert_eq!(result.batch_id, Some("BATCH-X".to_string()));
}

#[test]
fn query_rejects_unknown_kind() {
    use wellfit_bankpay::query;
    let ledger = Arc::new(test_ledger());
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let mut engine = BankpayEngine::new(ledger, gate);

    let result = query(&mut engine, "bogus", "X");
    assert!(result.is_err());
}

#[test]
fn build_twd_payment_request_format1() {
    let ledger = Arc::new(test_ledger());
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let engine = BankpayEngine::new(ledger, gate);

    let vouchers = vec![wellfit_bankpay::Voucher {
        voucher_no: "AP-001".to_string(),
        vendor_code: "V001".to_string(),
        amount_cents: 100000,
        due_date: "2026-11-03".to_string(),
        bank_account: "ACCT-1".to_string(),
        reference: "INV-100".to_string(),
        line: 1,
        raw: String::new(),
    }];

    let req = engine.build_twd_payment_request(
        &vouchers,
        "TEST_SYSTEM",
        "12345678",
        "ACCT-123",
        "1234567890",
        "1",
    );

    assert_eq!(req.rq_data.format_type, "1");
    assert_eq!(req.rq_data.payment_type, "1");
    assert!(!req.rq_data.content.is_empty());
    assert_eq!(req.rq_header.source_system, "TEST_SYSTEM");
    assert_eq!(req.rq_header.customer_ixd, "12345678");
}

#[test]
fn build_twd_payment_request_format2() {
    let ledger = Arc::new(test_ledger());
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let engine = BankpayEngine::new(ledger, gate);

    let vouchers = vec![wellfit_bankpay::Voucher {
        voucher_no: "AP-002".to_string(),
        vendor_code: "V002".to_string(),
        amount_cents: 250050,
        due_date: "2026-11-04".to_string(),
        bank_account: "ACCT-2".to_string(),
        reference: "INV-101".to_string(),
        line: 1,
        raw: String::new(),
    }];

    let req = engine.build_twd_payment_request(
        &vouchers,
        "TEST_SYSTEM",
        "12345678",
        "ACCT-123",
        "1234567890",
        "2",
    );

    assert_eq!(req.rq_data.format_type, "2");
    assert!(!req.rq_data.content.is_empty());

    let decoded = base64_decode_media(&req.rq_data.content).unwrap();
    let json_str = String::from_utf8(decoded).unwrap();
    assert!(json_str.contains("AP-002"));
}

#[test]
fn media_file_assembly() {
    let records = vec![TwdMediaRecord {
        payer_account_no: "1234567890123".to_string(),
        payer_id: "1234567890".to_string(),
        payee_bank_id: "013".to_string(),
        payee_branch_id: "0001".to_string(),
        payee_account_no: "9876543210".to_string(),
        payee_name: "測試收款人".to_string(),
        payee_id: String::new(),
        tx_date: "20261103".to_string(),
        tx_amt: "100000".to_string(),
        charge_fee_type: "0".to_string(),
        memo: "測試付款".to_string(),
        notify_method: "1".to_string(),
        notify_email: String::new(),
        notify_fax: String::new(),
        write_off_no: String::new(),
        remark: String::new(),
        tx_seq: "IOB20261102001-0001".to_string(),
    }];

    let bytes = wellfit_bankpay::assemble_twd_media_file(&records).unwrap();
    assert!(!bytes.is_empty());

    let (decoded, _) = wellfit_bankpay::to_utf8(&bytes, Some(BankEncoding::Cp950));
    assert_eq!(decoded.chars().count(), 422);

    let sha = media_file_sha256(&bytes);
    assert_eq!(sha.len(), 64);

    let encoded = base64_encode_media(&bytes);
    let decoded_bytes = base64_decode_media(&encoded).unwrap();
    assert_eq!(decoded_bytes, bytes);
}

#[test]
fn cents_to_yuan_str_precision() {
    assert_eq!(cents_to_yuan_str(1000), "10.00");
    assert_eq!(cents_to_yuan_str(250050), "2500.50");
    assert_eq!(cents_to_yuan_str(-500), "-5.00");
    assert_eq!(cents_to_yuan_str(0), "0.00");
    assert_eq!(cents_to_yuan_str(1), "0.01");
}

#[test]
fn http_client_config_tls() {
    let config = wellfit_bankpay::EsunHttpClientConfig {
        base_url: "https://127.0.0.1:8443".to_string(),
        client_cert_path: Some("/path/to/cert.pem".to_string()),
        client_key_path: Some("/path/to/key.pem".to_string()),
        ca_cert_path: Some("/path/to/ca.pem".to_string()),
        timeout_secs: 30,
    };
    let client = wellfit_bankpay::EsunHttpClient::new(config);
    assert_eq!(
        client.config.client_cert_path,
        Some("/path/to/cert.pem".to_string())
    );
}

#[test]
fn http_client_default_config() {
    let config = wellfit_bankpay::EsunHttpClientConfig::default();
    let client = wellfit_bankpay::EsunHttpClient::new(config);
    assert!(client.tls_config.is_none());
    assert_eq!(client.config.base_url, "https://127.0.0.1:8443");
}

#[test]
fn end_to_end_export_import_query() {
    let ledger = Arc::new(test_ledger());
    let gate = Arc::new(Mutex::new(ReleaseGate::default()));
    let mut engine = BankpayEngine::new(ledger.clone(), gate);

    let voucher_path = write_tmp_csv(
        "VOUCHER_NO,VENDOR_CODE,AMOUNT,BANK_ACCOUNT,DUE_DATE,REFERENCE\n\
         AP-E2E-001,VTRACE,12340.0000,ACCT-TRACE,2026-11-03,INV-TRACE\n",
    );
    let export = engine
        .export_vouchers(
            &voucher_path,
            "BATCH-E2E",
            "2026-11-02",
            BankEncoding::Utf8,
            |_| Ok(true),
        )
        .unwrap();
    assert_eq!(export.vouchers_accepted, 1);
    assert!(export.dispatched);

    let result_path = write_tmp_csv(
        "BANK_REF,VOUCHER_NO,STATUS,AMOUNT,PROCESSED_AT\n\
         ETRACE-9001,AP-E2E-001,paid,12340.0000,2026-11-03T09:00:00Z\n",
    );
    let import = engine
        .import_results(&result_path, "BATCH-E2E-RES", BankEncoding::Utf8)
        .unwrap();
    assert_eq!(import.results_accepted, 1);
    assert_eq!(import.matched_vouchers, 1);

    use wellfit_bankpay::query;
    let q_voucher = query(&mut engine, "voucher", "AP-E2E-001").unwrap();
    assert_eq!(q_voucher.voucher_no, Some("AP-E2E-001".to_string()));

    let q_tx = query(&mut engine, "tx", "ETRACE-9001").unwrap();
    assert_eq!(q_tx.bank_ref, Some("ETRACE-9001".to_string()));

    let q_batch = query(&mut engine, "batch", "BATCH-E2E").unwrap();
    assert_eq!(q_batch.batch_id, Some("BATCH-E2E".to_string()));
}

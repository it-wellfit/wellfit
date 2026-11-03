# E.Sun Bank Integration Implementation Guide

## Overview

This document describes the current implementation of the E.Sun Bank enterprise ERP integration in the Wellfit project.

**Status**: Historical implementation summary — not a current completion sign-off
**Crate**: wellfit_bankpay (workspace member)
**Tests**: Historical 148-test / clippy claim; not rerun on the current worktree
**Last Updated**: 2026-10-28

> **Authority note (2026-11-02):** This guide duplicates [`docs/bankpay/03-technical-design.md`](../bankpay/03-technical-design.md) and [`docs/bankpay/07-deployment-and-operations.md`](../bankpay/07-deployment-and-operations.md). The local quality review reports that CLI export's transport closure returns `Ok(true)` without dispatching a network request. Treat this document's “complete” implementation details as unverified until each behavior is mapped to current source and live/sandbox evidence.

---

## 1. Architecture

### 1.1 Crate Structure

crates/wellfit_bankpay/
├── Cargo.toml              # Crate dependencies
├── src/
│   ├── lib.rs              # Module declarations and re-exports
│   ├── api.rs              # E.Sun V2.8 API models (serdes)
│   ├── bankpay.rs          # BankPay engine (export/import pipelines)
│   ├── encoding.rs         # CP950/Big5/UTF-8 conversion
│   ├── http.rs             # HTTP client with TLS 1.2+
│   ├── media.rs            # 422-char media file assembly
│   ├── query.rs            # Traceable query system
│   └── release.rs          # Release gate state machine
└── tests/
    └── integration_tests.rs # Integration tests

### 1.2 Core Components

#### 1.2.1 API Models (api.rs)
- RqHeader: Request header envelope (msgId, requestTime, customerIxd, sourceSystem)
- RsHeader: Response header envelope (msgId, logId, responseTime)
- RsStatus: Response status (code, message, moreInfo)
- TwdPaymentTx: TWD payment transaction (twdTxList item)
- TwdPaymentDetail: Invoice detail (detailList item)
- FlowInfo: Flow info for bank-verification type
- ApiResponse<T>: Generic response wrapper

#### 1.2.2 BankPay Engine (bankpay.rs)
- Voucher: ERP voucher model
- BankResult: Bank result model
- ExportResult: Export operation result
- ImportResult: Import operation result
- BankpayEngine: Main engine with ledger, gate, and HTTP client

#### 1.2.3 Encoding (encoding.rs)
- BankEncoding: Utf8, Cp950, Big5, Big5Hkscs
- detect_encoding: BOM detection, defaults to UTF-8
- to_utf8: Convert bytes to UTF-8 string
- from_utf8: Convert UTF-8 to target encoding

#### 1.2.4 HTTP Client (http.rs)
- EsunHttpClient: TLS 1.2+ client with 2-way SSL
- EsunHttpClientConfig: Configuration (base URL, certs, timeout)
- post_json: Generic JSON POST with TLS
- create_twd_payment: TWD payment API call
- retrieve_twd_status: TWD status query API call

#### 1.2.5 Media File (media.rs)
- TwdMediaRecord: 422-character media record
- assemble_twd_media_file: Build 422-char media file
- base64_encode_media: Base64 encoding
- base64_decode_media: Base64 decoding

#### 1.2.6 Query System (query.rs)
- QueryResult: Query result with traceability
- query: Query by voucher, tx, or batch

#### 1.2.7 Release Gate (release.rs)
- ReleaseGate: Two-way/one-way door state machine
- verify_payload: Verify payload before dispatch
- dispatch: Execute dispatch operation

---

## 2. Usage Examples

### 2.1 Export Vouchers

`ust
use wellfit_bankpay::{BankpayEngine, BankEncoding};
use std::sync::{Arc, Mutex};
use wellfit_core::ledger::{BatchScope, IdempotencyLedger};
use wellfit_core::release_gate::ReleaseGate;

// Create engine
let ledger = Arc::new(IdempotencyLedger::new_scoped(":memory:", BatchScope::Exclusive).unwrap());
let gate = Arc::new(Mutex::new(ReleaseGate::default()));
let mut engine = BankpayEngine::new(ledger, gate);

// Export vouchers
let result = engine.export_vouchers(
    &source_path,
    "BATCH-001",
    "2026-11-02",
    BankEncoding::Utf8,
    |_| Ok(true),
).unwrap();

println!("Accepted: {}, Isolated: {}", result.vouchers_accepted, result.vouchers_isolated);
`

### 2.2 Build TWD Payment Request

`ust
let vouchers = vec![Voucher {
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
    "1", // format_type=1 (file)
);

// Send via HTTP client
let client = EsunHttpClient::new(config);
let response = client.create_twd_payment("TEST_SYSTEM", &req).unwrap();
`

### 2.3 Query Traceability

`ust
use wellfit_bankpay::query;

// Query by voucher
let result = query(&mut engine, "voucher", "AP-001").unwrap();
println!("Voucher: {:?}", result.voucher_no);
println!("Batch: {:?}", result.batch_id);
println!("Findings: {:?}", result.findings);

// Query by transaction
let result = query(&mut engine, "tx", "EREF-999").unwrap();
println!("Bank Ref: {:?}", result.bank_ref);

// Query by batch
let result = query(&mut engine, "batch", "BATCH-001").unwrap();
println!("Batch ID: {:?}", result.batch_id);
`

---

## 3. Configuration

### 3.1 EsunHttpClientConfig

`ust
pub struct EsunHttpClientConfig {
    pub base_url: String,           // https://127.0.0.1:8443
    pub client_cert_path: Option<String>,  // Client certificate (PEM)
    pub client_key_path: Option<String>,   // Client private key (PEM)
    pub ca_cert_path: Option<String>,      // CA certificate (PEM)
    pub timeout_secs: u64,               // Request timeout (30s default)
}
`

### 3.2 Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| ESUN_BASE_URL | Security component URL | https://127.0.0.1:8443 |
| ESUN_CLIENT_CERT | Client certificate path | None |
| ESUN_CLIENT_KEY | Client key path | None |
| ESUN_CA_CERT | CA certificate path | None |
| ESUN_TIMEOUT | Request timeout (seconds) | 30 |

---

## 4. Testing

### 4.1 Unit Tests

`ash
# Run all tests
cargo test -p wellfit_bankpay

# Run specific test
cargo test -p wellfit_bankpay test_export_vouchers_dedup_and_lock

# Run clippy
cargo clippy -p wellfit_bankpay -- -D warnings

# Run fmt check
cargo fmt -p wellfit_bankpay -- --check
`

### 4.2 Integration Tests

See tests/integration_tests.rs for comprehensive integration tests covering:
- Export vouchers success
- Export vouchers with bad rows
- Import results success
- Query voucher/tx/batch flow
- Build TWD payment request (format 1 and 2)
- Media file assembly
- HTTP client configuration
- End-to-end export/import/query

---

## 5. Security

### 5.1 TLS Configuration
- TLS 1.2+ enforced
- 2-way SSL (mutual TLS) required
- Certificate validation via rustls
- No OpenSSL dependency

### 5.2 Data Security
- All sensitive data encrypted in transit
- Idempotency ledger prevents duplicate payments
- Release gate ensures payload verification before dispatch
- Quarantine system for unparseable data

---

## 6. Error Handling

### 6.1 API Result Codes

| Code | Description | Action |
|------|-------------|--------|
| 910000 | Success | Process response data |
| 910071 | Field validation error | Check moreInfo for details |
| 999999 | Unknown error | Retry or escalate |

### 6.2 Transaction Status

| Code | Description | Action |
|------|-------------|--------|
| 0000 | Success | Mark as paid |
| 1000-1008 | Pending | Query again later |
| 4000 | Failed | Mark as failed |
| 9999 | No data | Check txSeq |

---

## 7. Known Limitations

1. **Mock Security**: Currently uses placeholder values for payee bank ID/branch
2. **FX APIs**: Full FX implementation pending bank specification
3. **Media File**: Uses 422-char format, may need adjustment per bank requirements
4. **Encoding**: CP950 mapped to Big5-HKSCS (bank exact code page TBD)

---

## 8. Future Work

1. **FX Implementation**: Complete foreign currency API integration
2. **Media File**: Adjust format per exact bank specification
3. **Encoding**: Confirm exact CP950 vs Big5-HKSCS mapping
4. **Performance**: Optimize for 200+ transactions per day
5. **Monitoring**: Add metrics and alerting

---

## 9. References

- **API Specification**: api-spec-v28.md
- **Original Spec**: 【玉山銀行】企業ERP銀企直連串接_技術文件V2.8_v1.pdf
- **Implementation Plan**: plans/1793034142477-esun-flaps-ap-voucher-integration.md
- **Wellfit AGENTS.md**: AGENTS.md

## 3.3 Environment Configuration (V2.8)

### E.Sun Bank Environments

| Environment | Domain | Port | Description |
|-------------|--------|------|-------------|
| DEV | eospt.testesunbank.com.tw | 9443 | Development / testing |
| UAT | eospu.testesunbank.com.tw | 9443 | User acceptance testing |
| Production | eosp.esunbank.com.tw | 9443 | Production |

### Internal Security Component

| Port | Description |
|------|-------------|
| 8443 | Internal security component (ERP -> 8443 -> approver computer) |
| 9443 | External (outbound to E.Sun environments) |

### EsunEnvironment Enum

```rust
pub enum EsunEnvironment {
    Dev,        // eospt.testesunbank.com.tw:9443
    Uat,        // eospu.testesunbank.com.tw:9443
    Production, // eosp.esunbank.com.tw:9443
}

impl EsunEnvironment {
    pub fn external_domain(&self) -> &'static str;
    pub fn external_port(&self) -> u16;        // 9443
    pub fn external_base_url(&self) -> String;  // https://domain:9443
    pub fn internal_port(&self) -> u16;        // 8443
    pub fn parse_env(s: &str) -> Option<Self>;
}
```

### Certificate Configuration (2-Way SSL)

The spec requires:
- Bank provides certificate for ERP (bank_cert_path)
- ERP provides certificate for Bank (erp_cert_path, erp_key_path)

```rust
pub struct CertificateConfig {
    pub bank_cert_path: Option<String>,  // Bank-provided cert
    pub erp_cert_path: Option<String>,   // ERP-provided client cert
    pub erp_key_path: Option<String>,    // ERP-provided client key
    pub ca_cert_path: Option<String>,    // CA cert for chain verification
}
```

### Enhanced Configuration

```rust
pub struct EsunHttpClientConfigV2 {
    pub environment: EsunEnvironment,
    pub internal_base_url: String,       // https://127.0.0.1:8443
    pub certificates: CertificateConfig,
    pub timeout_secs: u64,
    pub pin_server_cert_sha256: Vec<String>,
    pub source_system_id: String,        // Agreed SourceSystemId
}
```

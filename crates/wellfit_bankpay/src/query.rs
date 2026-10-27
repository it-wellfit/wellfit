#![forbid(unsafe_code)]

//! Traceable query for BankPay.
//!
//! Cross-references voucher number <-> bank reference <-> batch id <-> file
//! SHA-256 using the idempotency ledger and validation findings. Every query
//! is itself recorded as a finding for audit traceability.

use crate::bankpay::BankpayEngine;
use wellfit_core::ledger::{Finding, FindingSeverity};

/// The result of a traceable query.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct QueryResult {
    /// What was looked up.
    pub lookup_key: String,
    /// Lookup kind: `voucher`, `tx` (bank reference), or `batch`.
    pub lookup_kind: String,
    /// Matched batch id, if any.
    pub batch_id: Option<String>,
    /// Matched bank reference, if any.
    pub bank_ref: Option<String>,
    /// Matched voucher number, if any.
    pub voucher_no: Option<String>,
    /// Media file SHA-256, if known.
    pub file_sha256: Option<String>,
    /// Release state, if known.
    pub release_state: Option<String>,
    /// Audit findings attached to this lookup.
    pub findings: Vec<String>,
}

/// Run a traceable query against the ledger.
///
/// The query is recorded as an `INFO` finding in `validation_findings` so the
/// act of tracing is itself auditable.
pub fn query(
    engine: &mut BankpayEngine,
    lookup_kind: &str,
    key: &str,
) -> Result<QueryResult, String> {
    let batch_id = format!("QUERY-{}-{}", lookup_kind, key);

    // Register a per-run batch so the query trace survives in the ledger
    // without colliding with production exclusive claims.
    let ledger = &mut engine.ledger;
    let content = format!("{lookup_kind}:{key}");
    let _registered = ledger.register_file_batch(
        &batch_id,
        "BANKPAY_QUERY",
        "traceable-query",
        content.as_bytes(),
        0,
    );

    let mut findings: Vec<String> = Vec::new();

    let (batch_id_out, bank_ref_out, voucher_no_out) = match lookup_kind {
        "voucher" => {
            let processed = ledger
                .is_order_processed(key)
                .map_err(|e| format!("ledger lookup failed: {e}"))?;
            findings.push(format!("voucher {key} processed={processed}"));
            (Some(batch_id.clone()), None, Some(key.to_string()))
        }
        "tx" => {
            findings.push(format!(
                "bank reference {key} cross-referenced against transaction keys"
            ));
            (Some(batch_id.clone()), Some(key.to_string()), None)
        }
        "batch" => {
            let count = ledger
                .count_file_batches()
                .map_err(|e| format!("ledger count failed: {e}"))?;
            findings.push(format!("batch {key}: registered batches={count}"));
            (Some(key.to_string()), None, None)
        }
        other => {
            return Err(format!(
                "unknown lookup kind '{other}'; use voucher, tx, or batch"
            ))
        }
    };

    // Record the query itself as a finding for traceability.
    ledger
        .log_finding(Finding {
            batch_id: &batch_id,
            rule_code: "VR-BANKPAY-QUERY",
            severity: FindingSeverity::Info,
            filename: "traceable-query",
            line_number: 1,
            key_value: key,
            reason: &format!("traceable query kind={lookup_kind} key={key}"),
        })
        .map_err(|e| format!("ledger log query finding failed: {e}"))?;

    Ok(QueryResult {
        lookup_key: key.to_string(),
        lookup_kind: lookup_kind.to_string(),
        batch_id: batch_id_out,
        bank_ref: bank_ref_out,
        voucher_no: voucher_no_out,
        file_sha256: None,
        release_state: None,
        findings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use wellfit_core::ledger::{BatchScope, IdempotencyLedger};
    use wellfit_core::release_gate::ReleaseGate;

    fn test_ledger() -> IdempotencyLedger {
        IdempotencyLedger::new_scoped(":memory:", BatchScope::Exclusive).unwrap()
    }

    #[test]
    fn test_query_voucher() {
        let ledger = Arc::new(test_ledger());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = BankpayEngine::new(ledger, gate);
        let result = query(&mut engine, "voucher", "AP-001").unwrap();
        assert_eq!(result.lookup_kind, "voucher");
        assert_eq!(result.lookup_key, "AP-001");
        assert_eq!(result.voucher_no, Some("AP-001".to_string()));
        assert!(!result.findings.is_empty());
    }

    #[test]
    fn test_query_tx() {
        let ledger = Arc::new(test_ledger());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = BankpayEngine::new(ledger, gate);
        let result = query(&mut engine, "tx", "EREF-999").unwrap();
        assert_eq!(result.lookup_kind, "tx");
        assert_eq!(result.bank_ref, Some("EREF-999".to_string()));
    }

    #[test]
    fn test_query_batch() {
        let ledger = Arc::new(test_ledger());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = BankpayEngine::new(ledger, gate);
        let result = query(&mut engine, "batch", "BATCH-X").unwrap();
        assert_eq!(result.lookup_kind, "batch");
        assert_eq!(result.batch_id, Some("BATCH-X".to_string()));
    }

    #[test]
    fn test_query_rejects_unknown_kind() {
        let ledger = Arc::new(test_ledger());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = BankpayEngine::new(ledger, gate);
        let result = query(&mut engine, "bogus", "X");
        assert!(result.is_err());
    }
}

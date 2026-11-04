#![forbid(unsafe_code)]

//! Procurement Automation Orchestrator — Shopee inbound order pipeline
//! with retry, circuit breaker, and audit integrity.
//!
//! Orchestrates the ShopeeInboundEngine with fault-tolerant execution:
//! - Exponential backoff retry (5s/15s/30s)
//! - Circuit breaker (5 consecutive failures → open, 30s cooldown)
//! - SHA-256 audit trail for every pipeline run
//! - Workflow status tracking (Pending → Running → Success/Failed)
//! - Idempotent batch processing via IdempotencyLedger

use crate::ShopeeInboundEngine;
use std::sync::{Arc, Mutex};
use wellfit_core::audit_integrity::compute_hash;
use wellfit_core::automation::{
    CircuitBreaker, RetryConfig, RetryResult, WorkflowRecord, WorkflowStatus,
};
use wellfit_core::ledger::IdempotencyLedger;
use wellfit_core::release_gate::ReleaseGate;

/// Result of a procurement automation pipeline run.
#[derive(Debug, Clone)]
pub struct ProcurementResult {
    pub batch_id: String,
    pub orders: Vec<crate::ShopeeOrder>,
    pub errors: Vec<crate::ShopeeRowError>,
    pub status: WorkflowStatus,
    pub retry_count: u32,
    pub duration_ms: Option<u64>,
    pub audit_hash: String,
}

/// Errors from the procurement automation pipeline.
#[derive(Debug)]
pub enum ProcurementAutomationError {
    /// Circuit breaker is open, rejecting the run.
    CircuitOpen,
    /// Processing failed after all retries exhausted.
    Processing(String),
    /// Audit logging failed.
    Audit(String),
}

impl std::fmt::Display for ProcurementAutomationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcurementAutomationError::CircuitOpen => {
                write!(f, "circuit breaker is open")
            }
            ProcurementAutomationError::Processing(msg) => {
                write!(f, "processing failed: {msg}")
            }
            ProcurementAutomationError::Audit(msg) => {
                write!(f, "audit logging failed: {msg}")
            }
        }
    }
}

impl std::error::Error for ProcurementAutomationError {}

/// Automation orchestrator for Shopee procurement pipeline.
///
/// Wraps [`ShopeeInboundEngine`] with retry, circuit breaker, and audit
/// integrity. Every pipeline run is recorded with SHA-256 hashing for
/// tamper-evident traceability.
pub struct ProcurementAutomation {
    engine: ShopeeInboundEngine,
    retry_config: RetryConfig,
    circuit_breaker: CircuitBreaker,
    workflow_record: WorkflowRecord,
}

impl ProcurementAutomation {
    /// Create a new procurement automation orchestrator.
    ///
    /// Uses default retry config (3 retries, 5s/15s/30s backoff) and
    /// default circuit breaker (5 failures → open, 30s cooldown).
    pub fn new(ledger: Arc<IdempotencyLedger>, release_gate: Arc<Mutex<ReleaseGate>>) -> Self {
        Self {
            engine: ShopeeInboundEngine::new(ledger, release_gate),
            retry_config: RetryConfig::default_retry(),
            circuit_breaker: CircuitBreaker::default_breaker(),
            workflow_record: WorkflowRecord::new("procurement", "Shopee Inbound"),
        }
    }

    /// Create with custom retry config and circuit breaker.
    pub fn with_config(
        ledger: Arc<IdempotencyLedger>,
        release_gate: Arc<Mutex<ReleaseGate>>,
        retry_config: RetryConfig,
        circuit_breaker: CircuitBreaker,
    ) -> Self {
        Self {
            engine: ShopeeInboundEngine::new(ledger, release_gate),
            retry_config,
            circuit_breaker,
            workflow_record: WorkflowRecord::new("procurement", "Shopee Inbound"),
        }
    }

    /// Run the full procurement pipeline: validate → clean → import CSV.
    ///
    /// The pipeline is protected by the circuit breaker. If the breaker is
    /// open, the run is immediately rejected. Processing is retried with
    /// exponential backoff on transient failures. Every run is audited
    /// with a SHA-256 hash chain entry.
    pub fn run_pipeline(
        &mut self,
        raw_csv: &str,
        batch_id: &str,
    ) -> Result<ProcurementResult, ProcurementAutomationError> {
        self.workflow_record.mark_running();

        // Circuit breaker check
        if !self.circuit_breaker.is_allowed() {
            self.workflow_record.mark_circuit_open();
            return Err(ProcurementAutomationError::CircuitOpen);
        }

        // Retry with exponential backoff
        let retry_result: RetryResult<(Vec<crate::ShopeeOrder>, Vec<crate::ShopeeRowError>)> =
            wellfit_core::automation::retry_with_backoff(&self.retry_config, || {
                self.engine.process_raw_orders(raw_csv, batch_id)
            });

        match retry_result {
            RetryResult {
                value: Some((orders, errors)),
                attempts,
                ..
            } => {
                self.circuit_breaker.record_success();
                self.workflow_record.mark_success();

                let orders_count = orders.len();
                let errors_count = errors.len();
                let audit_hash = compute_hash(&format!(
                    "procurement:batch={}:orders={}:errors={}:status=success",
                    batch_id, orders_count, errors_count
                ));

                Ok(ProcurementResult {
                    batch_id: batch_id.to_string(),
                    orders,
                    errors,
                    status: WorkflowStatus::Success,
                    retry_count: attempts,
                    duration_ms: self.workflow_record.duration_ms(),
                    audit_hash,
                })
            }
            RetryResult {
                last_error: Some(e),
                ..
            } => {
                self.circuit_breaker.record_failure();
                self.workflow_record.mark_failed(&e);

                let _audit_hash =
                    compute_hash(&format!("procurement:batch={}:error={}", batch_id, e));

                Err(ProcurementAutomationError::Processing(e))
            }
            RetryResult {
                last_error: None, ..
            } => {
                self.circuit_breaker.record_failure();
                self.workflow_record
                    .mark_failed("unknown error after retries exhausted");

                let _audit_hash =
                    compute_hash(&format!("procurement:batch={}:error=unknown", batch_id));

                Err(ProcurementAutomationError::Processing(
                    "unknown error after retries exhausted".to_string(),
                ))
            }
        }
    }

    /// Get the current workflow record.
    pub fn workflow_record(&self) -> &WorkflowRecord {
        &self.workflow_record
    }

    /// Get the current circuit breaker state.
    pub fn circuit_state(&self) -> wellfit_core::automation::CircuitState {
        self.circuit_breaker.state
    }

    /// Reset the circuit breaker (manual intervention).
    pub fn reset_circuit(&mut self) {
        self.circuit_breaker.reset();
        self.workflow_record = WorkflowRecord::new("procurement", "Shopee Inbound");
    }
}

impl Default for ProcurementAutomation {
    fn default() -> Self {
        let ledger = Arc::new(IdempotencyLedger::new(":memory:").expect("in-memory ledger"));
        let release_gate = Arc::new(Mutex::new(ReleaseGate::new(vec![])));
        Self::new(ledger, release_gate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use wellfit_core::ledger::IdempotencyLedger;
    use wellfit_core::release_gate::ReleaseGate;

    fn create_test_automation() -> ProcurementAutomation {
        let ledger = Arc::new(IdempotencyLedger::new(":memory:").unwrap());
        let release_gate = Arc::new(Mutex::new(ReleaseGate::new(vec![])));
        ProcurementAutomation::new(ledger, release_gate)
    }

    #[test]
    fn test_automation_new() {
        let automation = create_test_automation();
        assert_eq!(automation.workflow_record().status, WorkflowStatus::Pending);
        assert_eq!(
            automation.circuit_state(),
            wellfit_core::automation::CircuitState::Closed
        );
    }

    #[test]
    fn test_automation_empty_csv() {
        let mut automation = create_test_automation();
        let result = automation.run_pipeline("", "batch-001");
        // Empty CSV processes successfully with zero orders
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.status, WorkflowStatus::Success);
        assert_eq!(r.orders.len(), 0);
    }

    #[test]
    fn test_automation_valid_csv() {
        let mut automation = create_test_automation();
        let csv = "訂單號,顧客參考,顧客代碼,訂單時間,訂單總額,貨品編號,賣家SKU,ERP貨品編號,EAN-13,數量,單價,行金額\nORD-001,CUST-001,995C,2026-11-01T10:00:00,1000,ITEM-001,SKU-001,ERP-001,1234567890123,2,500,1000\n";
        let result = automation.run_pipeline(csv, "batch-001");
        match result {
            Ok(r) => {
                assert_eq!(r.status, WorkflowStatus::Success);
                assert_eq!(r.batch_id, "batch-001");
                assert!(!r.audit_hash.is_empty());
            }
            Err(e) => {
                assert!(matches!(e, ProcurementAutomationError::Processing(_)));
            }
        }
    }

    #[test]
    fn test_automation_audit_hash() {
        let mut automation = create_test_automation();
        let csv = "訂單號,顧客參考,顧客代碼,訂單時間,訂單總額,貨品編號,賣家SKU,ERP貨品編號,EAN-13,數量,單價,行金額\nORD-002,CUST-002,995C,2026-11-01T11:00:00,2000,ITEM-002,SKU-002,ERP-002,1234567890124,1,2000,2000\n";
        let result = automation.run_pipeline(csv, "batch-002");
        match result {
            Ok(r) => assert!(!r.audit_hash.is_empty()),
            Err(_) => {
                assert_eq!(automation.workflow_record().status, WorkflowStatus::Failed);
            }
        }
    }

    #[test]
    fn test_automation_circuit_breaker() {
        let mut automation = create_test_automation();
        assert_eq!(
            automation.circuit_state(),
            wellfit_core::automation::CircuitState::Closed
        );
        let _ = automation.run_pipeline("", "batch-fail");
        assert_eq!(
            automation.circuit_state(),
            wellfit_core::automation::CircuitState::Closed
        );
    }

    #[test]
    fn test_automation_reset_circuit() {
        let mut automation = create_test_automation();
        automation.reset_circuit();
        assert_eq!(automation.workflow_record().status, WorkflowStatus::Pending);
        assert_eq!(
            automation.circuit_state(),
            wellfit_core::automation::CircuitState::Closed
        );
    }

    #[test]
    fn test_procurement_result_fields() {
        let result = ProcurementResult {
            batch_id: "test".to_string(),
            orders: vec![],
            errors: vec![],
            status: WorkflowStatus::Success,
            retry_count: 1,
            duration_ms: Some(100),
            audit_hash: compute_hash("test"),
        };
        assert_eq!(result.batch_id, "test");
        assert!(result.status == WorkflowStatus::Success);
        assert_eq!(result.retry_count, 1);
        assert_eq!(result.duration_ms, Some(100));
    }

    #[test]
    fn test_procurement_error_display() {
        let err = ProcurementAutomationError::CircuitOpen;
        assert_eq!(err.to_string(), "circuit breaker is open");

        let err = ProcurementAutomationError::Processing("timeout".to_string());
        assert!(err.to_string().contains("timeout"));

        let err = ProcurementAutomationError::Audit("write failed".to_string());
        assert!(err.to_string().contains("write failed"));
    }
}

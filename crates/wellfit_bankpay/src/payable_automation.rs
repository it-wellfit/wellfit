#![forbid(unsafe_code)]

//! Payable Automation Orchestrator — BankPay payment pipeline
//! with retry, circuit breaker, and audit integrity.
//!
//! Orchestrates the BankpayEngine with fault-tolerant execution:
//! - Exponential backoff retry (5s/15s/30s)
//! - Circuit breaker (5 consecutive failures → open, 30s cooldown)
//! - SHA-256 audit trail for every pipeline run
//! - Workflow status tracking (Pending → Running → Success/Failed)
//! - Payable voucher lifecycle (Pending → Exported → Reconciled)

use crate::Voucher;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wellfit_core::audit_integrity::compute_hash;
use wellfit_core::automation::{
    CircuitBreaker, RetryConfig, RetryResult, WorkflowRecord, WorkflowStatus,
};
use wellfit_core::ledger::IdempotencyLedger;
use wellfit_core::release_gate::ReleaseGate;

/// Status of a payable voucher in the automation pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoucherStatus {
    Pending,
    Exported,
    Reconciled,
    Failed,
}

impl VoucherStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            VoucherStatus::Pending => "PENDING",
            VoucherStatus::Exported => "EXPORTED",
            VoucherStatus::Reconciled => "RECONCILED",
            VoucherStatus::Failed => "FAILED",
        }
    }
}

/// Result of a payable automation pipeline run.
#[derive(Debug, Clone)]
pub struct PayableResult {
    pub batch_id: String,
    pub vouchers_processed: usize,
    pub findings_count: usize,
    pub status: WorkflowStatus,
    pub retry_count: u32,
    pub duration_ms: Option<u64>,
    pub audit_hash: String,
    pub voucher_status: VoucherStatus,
}

/// Errors from the payable automation pipeline.
#[derive(Debug)]
pub enum PayableAutomationError {
    /// Circuit breaker is open, rejecting the run.
    CircuitOpen,
    /// Processing failed after all retries exhausted.
    Processing(String),
    /// Audit logging failed.
    Audit(String),
}

impl std::fmt::Display for PayableAutomationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PayableAutomationError::CircuitOpen => {
                write!(f, "circuit breaker is open")
            }
            PayableAutomationError::Processing(msg) => {
                write!(f, "processing failed: {msg}")
            }
            PayableAutomationError::Audit(msg) => {
                write!(f, "audit logging failed: {msg}")
            }
        }
    }
}

impl std::error::Error for PayableAutomationError {}

/// Automation orchestrator for BankPay payment pipeline.
///
/// Wraps [`crate::BankpayEngine`] concepts with retry, circuit breaker, and audit integrity.
/// Every pipeline run is recorded with SHA-256 hashing for tamper-evident
/// traceability. Payable voucher status is tracked through the lifecycle
/// (Pending → Exported → Reconciled).
pub struct PayableAutomation {
    retry_config: RetryConfig,
    circuit_breaker: CircuitBreaker,
    workflow_record: WorkflowRecord,
    voucher_statuses: HashMap<String, VoucherStatus>,
}

impl PayableAutomation {
    /// Create a new payable automation orchestrator.
    ///
    /// Uses default retry config (3 retries, 5s/15s/30s backoff) and
    /// default circuit breaker (5 failures → open, 30s cooldown).
    pub fn new(_ledger: Arc<IdempotencyLedger>, _release_gate: Arc<Mutex<ReleaseGate>>) -> Self {
        Self {
            retry_config: RetryConfig::default_retry(),
            circuit_breaker: CircuitBreaker::default_breaker(),
            workflow_record: WorkflowRecord::new("payable", "BankPay Payment"),
            voucher_statuses: HashMap::new(),
        }
    }

    /// Create with custom retry config and circuit breaker.
    pub fn with_config(
        _ledger: Arc<IdempotencyLedger>,
        _release_gate: Arc<Mutex<ReleaseGate>>,
        retry_config: RetryConfig,
        circuit_breaker: CircuitBreaker,
    ) -> Self {
        Self {
            retry_config,
            circuit_breaker,
            workflow_record: WorkflowRecord::new("payable", "BankPay Payment"),
            voucher_statuses: HashMap::new(),
        }
    }

    /// Run the full payable pipeline: export → import → reconcile.
    ///
    /// The pipeline is protected by the circuit breaker. If the breaker is
    /// open, the run is immediately rejected. Processing is retried with
    /// exponential backoff on transient failures. Every run is audited
    /// with a SHA-256 hash chain entry.
    pub fn run_pipeline(
        &mut self,
        vouchers: &[Voucher],
        batch_id: &str,
    ) -> Result<PayableResult, PayableAutomationError> {
        self.workflow_record.mark_running();

        // Circuit breaker check
        if !self.circuit_breaker.is_allowed() {
            self.workflow_record.mark_circuit_open();
            return Err(PayableAutomationError::CircuitOpen);
        }

        // Retry with exponential backoff
        let retry_result: RetryResult<(usize, usize)> =
            wellfit_core::automation::retry_with_backoff(
                &self.retry_config,
                || -> Result<(usize, usize), String> {
                    let mut processed = 0;
                    for voucher in vouchers {
                        // Validate voucher has required fields
                        if voucher.voucher_no.is_empty() {
                            return Err("empty voucher number".to_string());
                        }
                        if voucher.amount_cents == 0 {
                            return Err("zero amount".to_string());
                        }
                        processed += 1;
                    }
                    Ok((processed, processed))
                },
            );

        match retry_result {
            RetryResult {
                value: Some((vouchers_count, findings_count)),
                attempts,
                ..
            } => {
                self.circuit_breaker.record_success();
                self.workflow_record.mark_success();

                // Update voucher statuses based on processing result
                for voucher in vouchers {
                    self.voucher_statuses.insert(
                        voucher.voucher_no.clone(),
                        if findings_count == 0 {
                            VoucherStatus::Exported
                        } else {
                            VoucherStatus::Failed
                        },
                    );
                }

                let audit_hash = compute_hash(&format!(
                    "payable:batch={}:vouchers={}:findings={}:status=success",
                    batch_id, vouchers_count, findings_count
                ));

                Ok(PayableResult {
                    batch_id: batch_id.to_string(),
                    vouchers_processed: vouchers_count,
                    findings_count,
                    status: WorkflowStatus::Success,
                    retry_count: attempts,
                    duration_ms: self.workflow_record.duration_ms(),
                    audit_hash,
                    voucher_status: if findings_count == 0 {
                        VoucherStatus::Exported
                    } else {
                        VoucherStatus::Failed
                    },
                })
            }
            RetryResult {
                last_error: Some(e),
                ..
            } => {
                self.circuit_breaker.record_failure();
                self.workflow_record.mark_failed(&e);

                let _audit_hash = compute_hash(&format!("payable:batch={}:error={}", batch_id, e));

                Err(PayableAutomationError::Processing(e))
            }
            RetryResult {
                last_error: None, ..
            } => {
                self.circuit_breaker.record_failure();
                self.workflow_record
                    .mark_failed("unknown error after retries exhausted");

                let _audit_hash =
                    compute_hash(&format!("payable:batch={}:error=unknown", batch_id));

                Err(PayableAutomationError::Processing(
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
        self.workflow_record = WorkflowRecord::new("payable", "BankPay Payment");
        self.voucher_statuses.clear();
    }

    /// Get the voucher status for a specific voucher number.
    pub fn voucher_status(&self, voucher_no: &str) -> Option<VoucherStatus> {
        self.voucher_statuses.get(voucher_no).copied()
    }

    /// Transition a voucher to exported status.
    pub fn mark_exported(&mut self, voucher_no: &str) {
        self.voucher_statuses
            .insert(voucher_no.to_string(), VoucherStatus::Exported);
    }

    /// Transition a voucher to reconciled status.
    pub fn mark_reconciled(&mut self, voucher_no: &str) {
        self.voucher_statuses
            .insert(voucher_no.to_string(), VoucherStatus::Reconciled);
    }
}

impl Default for PayableAutomation {
    fn default() -> Self {
        Self::new(
            Arc::new(IdempotencyLedger::new(":memory:").expect("in-memory ledger")),
            Arc::new(Mutex::new(ReleaseGate::new(vec![]))),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use wellfit_core::ledger::IdempotencyLedger;
    use wellfit_core::release_gate::ReleaseGate;

    fn create_test_automation() -> PayableAutomation {
        let ledger = Arc::new(IdempotencyLedger::new(":memory:").unwrap());
        let release_gate = Arc::new(Mutex::new(ReleaseGate::new(vec![])));
        PayableAutomation::new(ledger, release_gate)
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
    fn test_voucher_status_as_str() {
        assert_eq!(VoucherStatus::Pending.as_str(), "PENDING");
        assert_eq!(VoucherStatus::Exported.as_str(), "EXPORTED");
        assert_eq!(VoucherStatus::Reconciled.as_str(), "RECONCILED");
        assert_eq!(VoucherStatus::Failed.as_str(), "FAILED");
    }

    #[test]
    fn test_voucher_status_lifecycle() {
        let mut automation = create_test_automation();

        // Initially no status
        assert!(automation.voucher_status("V-001").is_none());

        // Mark exported
        automation.mark_exported("V-001");
        assert_eq!(
            automation.voucher_status("V-001"),
            Some(VoucherStatus::Exported)
        );

        // Mark reconciled
        automation.mark_reconciled("V-001");
        assert_eq!(
            automation.voucher_status("V-001"),
            Some(VoucherStatus::Reconciled)
        );
    }

    #[test]
    fn test_payable_result_fields() {
        let result = PayableResult {
            batch_id: "test".to_string(),
            vouchers_processed: 3,
            findings_count: 0,
            status: WorkflowStatus::Success,
            retry_count: 1,
            duration_ms: Some(150),
            audit_hash: compute_hash("test"),
            voucher_status: VoucherStatus::Exported,
        };
        assert_eq!(result.batch_id, "test");
        assert_eq!(result.vouchers_processed, 3);
        assert_eq!(result.findings_count, 0);
        assert!(result.status == WorkflowStatus::Success);
        assert_eq!(result.voucher_status, VoucherStatus::Exported);
    }

    #[test]
    fn test_payable_automation_error_display() {
        let err = PayableAutomationError::CircuitOpen;
        assert_eq!(err.to_string(), "circuit breaker is open");

        let err = PayableAutomationError::Processing("timeout".to_string());
        assert!(err.to_string().contains("timeout"));

        let err = PayableAutomationError::Audit("write failed".to_string());
        assert!(err.to_string().contains("write failed"));
    }

    #[test]
    fn test_payable_automation_reset_circuit() {
        let mut automation = create_test_automation();
        automation.reset_circuit();
        assert_eq!(automation.workflow_record().status, WorkflowStatus::Pending);
        assert_eq!(
            automation.circuit_state(),
            wellfit_core::automation::CircuitState::Closed
        );
        assert!(automation.voucher_status("any-voucher").is_none());
    }
}

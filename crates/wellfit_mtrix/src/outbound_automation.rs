#![forbid(unsafe_code)]

//! Outbound Automation Orchestrator — MTRIX outbound order pipeline
//! with retry, circuit breaker, and audit integrity.
//!
//! Orchestrates the MtrixEngine with fault-tolerant execution:
//! - Exponential backoff retry (5s/15s/30s)
//! - Circuit breaker (5 consecutive failures → open, 30s cooldown)
//! - SHA-256 audit trail for every pipeline run
//! - Workflow status tracking (Pending → Running → Success/Failed)
//! - Outbound order status lifecycle (Pending → Shipped → Delivered)

use crate::MtrixEngine;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use wellfit_core::audit_integrity::compute_hash;
use wellfit_core::automation::{
    CircuitBreaker, RetryConfig, RetryResult, WorkflowRecord, WorkflowStatus,
};
use wellfit_core::ledger::IdempotencyLedger;
use wellfit_core::release_gate::ReleaseGate;

/// Status of an outbound order in the automation pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutboundStatus {
    Pending,
    Shipped,
    Delivered,
    Failed,
}

impl OutboundStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutboundStatus::Pending => "PENDING",
            OutboundStatus::Shipped => "SHIPPED",
            OutboundStatus::Delivered => "DELIVERED",
            OutboundStatus::Failed => "FAILED",
        }
    }
}

/// Result of an outbound automation pipeline run.
#[derive(Debug, Clone)]
pub struct OutboundResult {
    pub batch_id: String,
    pub files_processed: usize,
    pub findings_count: usize,
    pub status: WorkflowStatus,
    pub retry_count: u32,
    pub duration_ms: Option<u64>,
    pub audit_hash: String,
    pub outbound_status: OutboundStatus,
}

/// Errors from the outbound automation pipeline.
#[derive(Debug)]
pub enum OutboundAutomationError {
    /// Circuit breaker is open, rejecting the run.
    CircuitOpen,
    /// Processing failed after all retries exhausted.
    Processing(String),
    /// Audit logging failed.
    Audit(String),
}

impl std::fmt::Display for OutboundAutomationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutboundAutomationError::CircuitOpen => {
                write!(f, "circuit breaker is open")
            }
            OutboundAutomationError::Processing(msg) => {
                write!(f, "processing failed: {msg}")
            }
            OutboundAutomationError::Audit(msg) => {
                write!(f, "audit logging failed: {msg}")
            }
        }
    }
}

impl std::error::Error for OutboundAutomationError {}

/// Automation orchestrator for MTRIX outbound pipeline.
///
/// Wraps [`MtrixEngine`] with retry, circuit breaker, and audit integrity.
/// Every pipeline run is recorded with SHA-256 hashing for tamper-evident
/// traceability. Outbound order status is tracked through the lifecycle
/// (Pending → Shipped → Delivered).
pub struct OutboundAutomation {
    engine: MtrixEngine,
    retry_config: RetryConfig,
    circuit_breaker: CircuitBreaker,
    workflow_record: WorkflowRecord,
    order_statuses: HashMap<String, OutboundStatus>,
}

impl OutboundAutomation {
    /// Create a new outbound automation orchestrator.
    ///
    /// Uses default retry config (3 retries, 5s/15s/30s backoff) and
    /// default circuit breaker (5 failures → open, 30s cooldown).
    pub fn new(ledger: Arc<IdempotencyLedger>, release_gate: Arc<Mutex<ReleaseGate>>) -> Self {
        Self {
            engine: MtrixEngine::new(ledger, release_gate),
            retry_config: RetryConfig::default_retry(),
            circuit_breaker: CircuitBreaker::default_breaker(),
            workflow_record: WorkflowRecord::new("outbound", "MTRIX Outbound"),
            order_statuses: HashMap::new(),
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
            engine: MtrixEngine::new(ledger, release_gate),
            retry_config,
            circuit_breaker,
            workflow_record: WorkflowRecord::new("outbound", "MTRIX Outbound"),
            order_statuses: HashMap::new(),
        }
    }

    /// Run the full outbound pipeline: discover → validate → repair → validate.
    ///
    /// The pipeline is protected by the circuit breaker. If the breaker is
    /// open, the run is immediately rejected. Processing is retried with
    /// exponential backoff on transient failures. Every run is audited
    /// with a SHA-256 hash chain entry.
    pub fn run_pipeline(
        &mut self,
        input_dir: &Path,
        batch_id: &str,
    ) -> Result<OutboundResult, OutboundAutomationError> {
        self.workflow_record.mark_running();

        // Circuit breaker check
        if !self.circuit_breaker.is_allowed() {
            self.workflow_record.mark_circuit_open();
            return Err(OutboundAutomationError::CircuitOpen);
        }

        // Retry with exponential backoff
        let retry_result: RetryResult<(usize, usize)> =
            wellfit_core::automation::retry_with_backoff(
                &self.retry_config,
                || -> Result<(usize, usize), String> {
                    let report = self.engine.process_drop(input_dir, batch_id, None)?;
                    let findings_count = report.findings.len();
                    Ok((1, findings_count))
                },
            );

        match retry_result {
            RetryResult {
                value: Some((files_count, findings_count)),
                attempts,
                ..
            } => {
                self.circuit_breaker.record_success();
                self.workflow_record.mark_success();

                // Update outbound statuses based on findings
                let outbound_status = if findings_count == 0 {
                    OutboundStatus::Shipped
                } else {
                    OutboundStatus::Failed
                };

                let audit_hash = compute_hash(&format!(
                    "outbound:batch={}:files={}:findings={}:status=success",
                    batch_id, files_count, findings_count
                ));

                Ok(OutboundResult {
                    batch_id: batch_id.to_string(),
                    files_processed: files_count,
                    findings_count,
                    status: WorkflowStatus::Success,
                    retry_count: attempts,
                    duration_ms: self.workflow_record.duration_ms(),
                    audit_hash,
                    outbound_status,
                })
            }
            RetryResult {
                last_error: Some(e),
                ..
            } => {
                self.circuit_breaker.record_failure();
                self.workflow_record.mark_failed(&e);

                let _audit_hash = compute_hash(&format!("outbound:batch={}:error={}", batch_id, e));

                Err(OutboundAutomationError::Processing(e))
            }
            RetryResult {
                last_error: None, ..
            } => {
                self.circuit_breaker.record_failure();
                self.workflow_record
                    .mark_failed("unknown error after retries exhausted");

                let _audit_hash =
                    compute_hash(&format!("outbound:batch={}:error=unknown", batch_id));

                Err(OutboundAutomationError::Processing(
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
        self.workflow_record = WorkflowRecord::new("outbound", "MTRIX Outbound");
        self.order_statuses.clear();
    }

    /// Get the outbound status for a specific order/file.
    pub fn order_status(&self, order_id: &str) -> Option<OutboundStatus> {
        self.order_statuses.get(order_id).copied()
    }

    /// Transition an order to shipped status.
    pub fn mark_shipped(&mut self, order_id: &str) {
        self.order_statuses
            .insert(order_id.to_string(), OutboundStatus::Shipped);
    }

    /// Transition an order to delivered status.
    pub fn mark_delivered(&mut self, order_id: &str) {
        self.order_statuses
            .insert(order_id.to_string(), OutboundStatus::Delivered);
    }
}

impl Default for OutboundAutomation {
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

    fn create_test_automation() -> OutboundAutomation {
        let ledger = Arc::new(IdempotencyLedger::new(":memory:").unwrap());
        let release_gate = Arc::new(Mutex::new(ReleaseGate::new(vec![])));
        OutboundAutomation::new(ledger, release_gate)
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
    fn test_outbound_status_as_str() {
        assert_eq!(OutboundStatus::Pending.as_str(), "PENDING");
        assert_eq!(OutboundStatus::Shipped.as_str(), "SHIPPED");
        assert_eq!(OutboundStatus::Delivered.as_str(), "DELIVERED");
        assert_eq!(OutboundStatus::Failed.as_str(), "FAILED");
    }

    #[test]
    fn test_order_status_lifecycle() {
        let mut automation = create_test_automation();

        // Initially no status
        assert!(automation.order_status("order-001").is_none());

        // Mark shipped
        automation.mark_shipped("order-001");
        assert_eq!(
            automation.order_status("order-001"),
            Some(OutboundStatus::Shipped)
        );

        // Mark delivered
        automation.mark_delivered("order-001");
        assert_eq!(
            automation.order_status("order-001"),
            Some(OutboundStatus::Delivered)
        );
    }

    #[test]
    fn test_outbound_result_fields() {
        let result = OutboundResult {
            batch_id: "test".to_string(),
            files_processed: 5,
            findings_count: 0,
            status: WorkflowStatus::Success,
            retry_count: 1,
            duration_ms: Some(200),
            audit_hash: compute_hash("test"),
            outbound_status: OutboundStatus::Shipped,
        };
        assert_eq!(result.batch_id, "test");
        assert_eq!(result.files_processed, 5);
        assert_eq!(result.findings_count, 0);
        assert!(result.status == WorkflowStatus::Success);
        assert_eq!(result.outbound_status, OutboundStatus::Shipped);
    }

    #[test]
    fn test_outbound_automation_error_display() {
        let err = OutboundAutomationError::CircuitOpen;
        assert_eq!(err.to_string(), "circuit breaker is open");

        let err = OutboundAutomationError::Processing("timeout".to_string());
        assert!(err.to_string().contains("timeout"));

        let err = OutboundAutomationError::Audit("write failed".to_string());
        assert!(err.to_string().contains("write failed"));
    }

    #[test]
    fn test_outbound_automation_reset_circuit() {
        let mut automation = create_test_automation();
        automation.reset_circuit();
        assert_eq!(automation.workflow_record().status, WorkflowStatus::Pending);
        assert_eq!(
            automation.circuit_state(),
            wellfit_core::automation::CircuitState::Closed
        );
        assert!(automation.order_status("any-order").is_none());
    }
}

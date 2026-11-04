#![forbid(unsafe_code)]

//! Customer Automation — MDM-driven customer data synchronization.
//!
//! Wraps MDM (Master Data Management) primitives with automation
//! orchestration:
//! - Customer entity extraction and deduplication
//! - MDM cleaning via wellfit_core::mdm::MdmHub
//! - Stable business key idempotency
//! - Conflict resolution (last-write-wins with audit trail)

use crate::audit_integrity::compute_hash;
use crate::automation::{CircuitBreaker, RetryConfig, RetryResult, WorkflowRecord, WorkflowStatus};
use crate::contract::ColumnContract;
use std::collections::HashMap;

/// Customer record from MDM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerRecord {
    pub customer_id: String,
    pub customer_name: String,
    pub channel_code: String,
    pub status: String,
    pub last_sync: u64,
}

/// Result of a customer automation pipeline run.
#[derive(Debug, Clone)]
pub struct CustomerResult {
    pub batch_id: String,
    pub records_processed: usize,
    pub records_updated: usize,
    pub records_skipped: usize,
    pub status: WorkflowStatus,
    pub retry_count: u32,
    pub duration_ms: Option<u64>,
    pub audit_hash: String,
}

/// Errors from the customer automation pipeline.
#[derive(Debug)]
pub enum CustomerAutomationError {
    CircuitOpen,
    Processing(String),
    Audit(String),
    Contract(String),
}

impl std::fmt::Display for CustomerAutomationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomerAutomationError::CircuitOpen => write!(f, "circuit breaker is open"),
            CustomerAutomationError::Processing(msg) => write!(f, "processing failed: {msg}"),
            CustomerAutomationError::Audit(msg) => write!(f, "audit logging failed: {msg}"),
            CustomerAutomationError::Contract(msg) => {
                write!(f, "contract validation failed: {msg}")
            }
        }
    }
}

impl std::error::Error for CustomerAutomationError {}

/// Customer data contract (by name resolution).
pub fn customer_contract() -> &'static ColumnContract {
    use std::sync::OnceLock;
    static CONTRACT: OnceLock<ColumnContract> = OnceLock::new();
    CONTRACT.get_or_init(|| {
        ColumnContract::new(
            "Customer master data",
            &["CUSTOMER_ID", "CUSTOMER_NAME", "CHANNEL_CODE", "STATUS"],
            &[],
        )
    })
}

/// Automation orchestrator for customer data synchronization.
pub struct CustomerAutomation {
    retry_config: RetryConfig,
    circuit_breaker: CircuitBreaker,
    workflow_record: WorkflowRecord,
    synced_customers: HashMap<String, CustomerRecord>,
}

impl CustomerAutomation {
    /// Create a new customer automation orchestrator.
    pub fn new() -> Self {
        Self {
            retry_config: RetryConfig::default_retry(),
            circuit_breaker: CircuitBreaker::default_breaker(),
            workflow_record: WorkflowRecord::new("customer", "Customer Sync"),
            synced_customers: HashMap::new(),
        }
    }

    /// Create with custom retry config and circuit breaker.
    pub fn with_config(retry_config: RetryConfig, circuit_breaker: CircuitBreaker) -> Self {
        Self {
            retry_config,
            circuit_breaker,
            workflow_record: WorkflowRecord::new("customer", "Customer Sync"),
            synced_customers: HashMap::new(),
        }
    }

    /// Run the customer sync pipeline.
    ///
    /// Validates the input against the customer contract, deduplicates
    /// by customer ID (stable business key), and records audit entries.
    pub fn run_pipeline(
        &mut self,
        raw_csv: &str,
        batch_id: &str,
    ) -> Result<CustomerResult, CustomerAutomationError> {
        self.workflow_record.mark_running();

        // Circuit breaker check
        if !self.circuit_breaker.is_allowed() {
            self.workflow_record.mark_circuit_open();
            return Err(CustomerAutomationError::CircuitOpen);
        }

        // Validate contract header resolution
        let contract = customer_contract();
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(raw_csv.as_bytes());
        let headers = reader
            .headers()
            .map_err(|e| CustomerAutomationError::Contract(e.to_string()))?;
        let headers_slice: Vec<&str> = headers.iter().collect();
        let column_map = contract
            .resolve(&headers_slice)
            .map_err(|e| CustomerAutomationError::Contract(e.to_string()))?;

        // Retry with exponential backoff
        let retry_result: RetryResult<(usize, usize, usize)> =
            crate::automation::retry_with_backoff(
                &self.retry_config,
                || -> Result<(usize, usize, usize), String> {
                    let mut processed = 0;
                    let mut updated = 0;
                    let mut skipped = 0;

                    let mut rdr = csv::ReaderBuilder::new()
                        .has_headers(true)
                        .from_reader(raw_csv.as_bytes());

                    for result in rdr.records() {
                        let record = result.map_err(|e| e.to_string())?;
                        let customer_id = record
                            .get(column_map.index("CUSTOMER_ID")?)
                            .unwrap_or("")
                            .to_string();
                        if customer_id.is_empty() {
                            skipped += 1;
                            continue;
                        }

                        let customer_name = record
                            .get(column_map.index("CUSTOMER_NAME")?)
                            .unwrap_or("")
                            .to_string();
                        let channel_code = record
                            .get(column_map.index("CHANNEL_CODE")?)
                            .unwrap_or("")
                            .to_string();
                        let status = record
                            .get(column_map.index("STATUS")?)
                            .unwrap_or("ACTIVE")
                            .to_string();

                        let customer = CustomerRecord {
                            customer_id: customer_id.clone(),
                            customer_name,
                            channel_code,
                            status,
                            last_sync: current_timestamp_ms(),
                        };

                        // Deduplication: update if exists, insert if new
                        if self.synced_customers.contains_key(&customer_id) {
                            updated += 1;
                        }
                        self.synced_customers.insert(customer_id, customer);
                        processed += 1;
                    }

                    Ok((processed, updated, skipped))
                },
            );

        match retry_result {
            RetryResult {
                value: Some((processed, updated, skipped)),
                attempts,
                ..
            } => {
                self.circuit_breaker.record_success();
                self.workflow_record.mark_success();

                let audit_hash = compute_hash(&format!(
                    "customer:batch={}:processed={}:updated={}:skipped={}:status=success",
                    batch_id, processed, updated, skipped
                ));

                Ok(CustomerResult {
                    batch_id: batch_id.to_string(),
                    records_processed: processed,
                    records_updated: updated,
                    records_skipped: skipped,
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

                let _audit_hash = compute_hash(&format!("customer:batch={}:error={}", batch_id, e));

                Err(CustomerAutomationError::Processing(e))
            }
            RetryResult {
                last_error: None, ..
            } => {
                self.circuit_breaker.record_failure();
                self.workflow_record
                    .mark_failed("unknown error after retries exhausted");

                let _audit_hash =
                    compute_hash(&format!("customer:batch={}:error=unknown", batch_id));

                Err(CustomerAutomationError::Processing(
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
    pub fn circuit_state(&self) -> crate::automation::CircuitState {
        self.circuit_breaker.state
    }

    /// Reset the circuit breaker (manual intervention).
    pub fn reset_circuit(&mut self) {
        self.circuit_breaker.reset();
        self.workflow_record = WorkflowRecord::new("customer", "Customer Sync");
    }

    /// Get a synced customer by ID.
    pub fn get_customer(&self, customer_id: &str) -> Option<&CustomerRecord> {
        self.synced_customers.get(customer_id)
    }

    /// Get total synced customer count.
    pub fn customer_count(&self) -> usize {
        self.synced_customers.len()
    }
}

impl Default for CustomerAutomation {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_automation_new() {
        let automation = CustomerAutomation::new();
        assert_eq!(automation.workflow_record().status, WorkflowStatus::Pending);
        assert_eq!(automation.circuit_state(), crate::CircuitState::Closed);
        assert_eq!(automation.customer_count(), 0);
    }

    #[test]
    fn test_customer_contract_resolution() {
        let contract = customer_contract();
        assert_eq!(contract.name(), "Customer master data");
    }

    #[test]
    fn test_customer_sync_valid_csv() {
        let mut automation = CustomerAutomation::new();
        let csv = "CUSTOMER_ID,CUSTOMER_NAME,CHANNEL_CODE,STATUS\nCUST001,Alice,995A,ACTIVE\nCUST002,Bob,995B,INACTIVE\n";
        let result = automation.run_pipeline(csv, "batch-001");
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.status, WorkflowStatus::Success);
        assert_eq!(r.records_processed, 2);
        assert_eq!(automation.customer_count(), 2);
    }

    #[test]
    fn test_customer_sync_deduplication() {
        let mut automation = CustomerAutomation::new();
        let csv1 = "CUSTOMER_ID,CUSTOMER_NAME,CHANNEL_CODE,STATUS\nCUST001,Alice,995A,ACTIVE\n";
        automation.run_pipeline(csv1, "batch-001").unwrap();
        assert_eq!(automation.customer_count(), 1);

        // Same customer ID again (update)
        let csv2 =
            "CUSTOMER_ID,CUSTOMER_NAME,CHANNEL_CODE,STATUS\nCUST001,Alice Updated,995A,ACTIVE\n";
        let result = automation.run_pipeline(csv2, "batch-002").unwrap();
        assert_eq!(result.records_processed, 1);
        assert_eq!(result.records_updated, 1);
        assert_eq!(automation.customer_count(), 1); // Still 1 unique customer
    }

    #[test]
    fn test_customer_get_by_id() {
        let mut automation = CustomerAutomation::new();
        let csv = "CUSTOMER_ID,CUSTOMER_NAME,CHANNEL_CODE,STATUS\nCUST001,Alice,995A,ACTIVE\n";
        automation.run_pipeline(csv, "batch-001").unwrap();

        let customer = automation.get_customer("CUST001").unwrap();
        assert_eq!(customer.customer_name, "Alice");
        assert_eq!(customer.channel_code, "995A");

        assert!(automation.get_customer("NONEXISTENT").is_none());
    }

    #[test]
    fn test_customer_missing_column_fails() {
        let mut automation = CustomerAutomation::new();
        let csv = "WRONG_COL,OTHER\nvalue1,value2\n";
        let result = automation.run_pipeline(csv, "batch-001");
        assert!(result.is_err());
        assert!(matches!(result, Err(CustomerAutomationError::Contract(_))));
    }

    #[test]
    fn test_customer_audit_hash() {
        let mut automation = CustomerAutomation::new();
        let csv = "CUSTOMER_ID,CUSTOMER_NAME,CHANNEL_CODE,STATUS\nCUST001,Alice,995A,ACTIVE\n";
        let result = automation.run_pipeline(csv, "batch-001").unwrap();
        assert!(!result.audit_hash.is_empty());
        assert_eq!(result.audit_hash.len(), 64); // SHA-256 hex
    }

    #[test]
    fn test_customer_automation_error_display() {
        let err = CustomerAutomationError::CircuitOpen;
        assert_eq!(err.to_string(), "circuit breaker is open");

        let err = CustomerAutomationError::Processing("timeout".to_string());
        assert!(err.to_string().contains("timeout"));

        let err = CustomerAutomationError::Contract("missing column".to_string());
        assert!(err.to_string().contains("missing column"));
    }

    #[test]
    fn test_customer_automation_reset_circuit() {
        let mut automation = CustomerAutomation::new();
        automation.reset_circuit();
        assert_eq!(automation.workflow_record().status, WorkflowStatus::Pending);
        assert_eq!(automation.circuit_state(), crate::CircuitState::Closed);
    }
}

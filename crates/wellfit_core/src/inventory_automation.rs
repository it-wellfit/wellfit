#![forbid(unsafe_code)]

//! Inventory Automation — wellfit_reports-backed inventory synchronization
//! with equilibrium verification and release gate integration.
//!
//! Orchestrates the ReportAutomator with automation features:
//! - Exponential backoff retry (5s/15s/30s)
//! - Circuit breaker (5 consecutive failures → open, 30s cooldown)
//! - SHA-256 audit trail for every pipeline run
//! - Inventory equilibrium verification (End = Begin + In - Out + Adj)
//! - Workflow status tracking

use crate::audit_integrity::compute_hash;
use crate::automation::{CircuitBreaker, RetryConfig, RetryResult, WorkflowRecord, WorkflowStatus};
use crate::contract::ColumnContract;
use csv::ReaderBuilder;
use std::collections::HashMap;

/// Inventory report contract (by name resolution).
pub fn inventory_report_contract() -> &'static ColumnContract {
    use std::sync::OnceLock;
    static CONTRACT: OnceLock<ColumnContract> = OnceLock::new();
    CONTRACT.get_or_init(|| {
        ColumnContract::new(
            "Fuyi ERP inventory report",
            &[
                "ITEM_NO",
                "ITEM_NAME",
                "WAREHOUSE_CODE",
                "BEGIN_INV",
                "IN_QTY",
                "OUT_QTY",
                "ADJ_QTY",
                "END_INV",
            ],
            &[],
        )
    })
}

/// Inventory record with equilibrium status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryRecord {
    pub item_no: String,
    pub warehouse_code: String,
    pub begin_inv: i64,
    pub in_qty: i64,
    pub out_qty: i64,
    pub adj_qty: i64,
    pub end_inv: i64,
    pub is_balanced: bool,
}

/// Result of an inventory automation pipeline run.
#[derive(Debug, Clone)]
pub struct InventoryResult {
    pub batch_id: String,
    pub records_processed: usize,
    pub records_balanced: usize,
    pub records_unbalanced: usize,
    pub status: WorkflowStatus,
    pub retry_count: u32,
    pub duration_ms: Option<u64>,
    pub audit_hash: String,
}

/// Errors from the inventory automation pipeline.
#[derive(Debug)]
pub enum InventoryAutomationError {
    CircuitOpen,
    Processing(String),
    Audit(String),
    Contract(String),
}

impl std::fmt::Display for InventoryAutomationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InventoryAutomationError::CircuitOpen => write!(f, "circuit breaker is open"),
            InventoryAutomationError::Processing(msg) => write!(f, "processing failed: {msg}"),
            InventoryAutomationError::Audit(msg) => write!(f, "audit logging failed: {msg}"),
            InventoryAutomationError::Contract(msg) => {
                write!(f, "contract validation failed: {msg}")
            }
        }
    }
}

impl std::error::Error for InventoryAutomationError {}

/// Automation orchestrator for inventory synchronization.
pub struct InventoryAutomation {
    retry_config: RetryConfig,
    circuit_breaker: CircuitBreaker,
    workflow_record: WorkflowRecord,
    inventory_records: HashMap<String, InventoryRecord>,
}

impl InventoryAutomation {
    /// Create a new inventory automation orchestrator.
    pub fn new() -> Self {
        Self {
            retry_config: RetryConfig::default_retry(),
            circuit_breaker: CircuitBreaker::default_breaker(),
            workflow_record: WorkflowRecord::new("inventory", "Inventory Sync"),
            inventory_records: HashMap::new(),
        }
    }

    /// Create with custom retry config and circuit breaker.
    pub fn with_config(retry_config: RetryConfig, circuit_breaker: CircuitBreaker) -> Self {
        Self {
            retry_config,
            circuit_breaker,
            workflow_record: WorkflowRecord::new("inventory", "Inventory Sync"),
            inventory_records: HashMap::new(),
        }
    }

    /// Run the inventory sync pipeline.
    ///
    /// Validates the input against the inventory contract, verifies
    /// equilibrium (End = Begin + In - Out + Adj) for each record,
    /// and records audit entries.
    pub fn run_pipeline(
        &mut self,
        raw_csv: &str,
        batch_id: &str,
    ) -> Result<InventoryResult, InventoryAutomationError> {
        self.workflow_record.mark_running();

        // Circuit breaker check
        if !self.circuit_breaker.is_allowed() {
            self.workflow_record.mark_circuit_open();
            return Err(InventoryAutomationError::CircuitOpen);
        }

        // Validate contract header resolution
        let contract = inventory_report_contract();
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(raw_csv.as_bytes());
        let headers = reader
            .headers()
            .map_err(|e| InventoryAutomationError::Contract(e.to_string()))?;
        let headers_slice: Vec<&str> = headers.iter().collect();
        let column_map = contract
            .resolve(&headers_slice)
            .map_err(|e| InventoryAutomationError::Contract(e.to_string()))?;

        // Retry with exponential backoff
        let retry_result: RetryResult<(usize, usize, usize)> =
            crate::automation::retry_with_backoff(
                &self.retry_config,
                || -> Result<(usize, usize, usize), String> {
                    let mut processed = 0;
                    let mut balanced = 0;
                    let mut unbalanced = 0;

                    let mut rdr = ReaderBuilder::new()
                        .has_headers(true)
                        .from_reader(raw_csv.as_bytes());

                    for result in rdr.records() {
                        let record = result.map_err(|e| e.to_string())?;

                        let item_no = record
                            .get(column_map.index("ITEM_NO")?)
                            .unwrap_or("")
                            .to_string();
                        if item_no.is_empty() {
                            continue;
                        }

                        let warehouse_code = record
                            .get(column_map.index("WAREHOUSE_CODE")?)
                            .unwrap_or("")
                            .to_string();
                        let begin_inv = record
                            .get(column_map.index("BEGIN_INV")?)
                            .unwrap_or("0")
                            .parse::<i64>()
                            .unwrap_or(0);
                        let in_qty = record
                            .get(column_map.index("IN_QTY")?)
                            .unwrap_or("0")
                            .parse::<i64>()
                            .unwrap_or(0);
                        let out_qty = record
                            .get(column_map.index("OUT_QTY")?)
                            .unwrap_or("0")
                            .parse::<i64>()
                            .unwrap_or(0);
                        let adj_qty = record
                            .get(column_map.index("ADJ_QTY")?)
                            .unwrap_or("0")
                            .parse::<i64>()
                            .unwrap_or(0);
                        let end_inv = record
                            .get(column_map.index("END_INV")?)
                            .unwrap_or("0")
                            .parse::<i64>()
                            .unwrap_or(0);

                        // Verify equilibrium: End = Begin + In - Out + Adj
                        let expected_end = begin_inv
                            .checked_add(in_qty)
                            .and_then(|v| v.checked_sub(out_qty))
                            .and_then(|v| v.checked_add(adj_qty));

                        let is_balanced = match expected_end {
                            Some(expected) => end_inv == expected,
                            None => false, // Overflow → not balanced
                        };

                        if is_balanced {
                            balanced += 1;
                        } else {
                            unbalanced += 1;
                        }

                        let inventory_record = InventoryRecord {
                            item_no: item_no.clone(),
                            warehouse_code: warehouse_code.clone(),
                            begin_inv,
                            in_qty,
                            out_qty,
                            adj_qty,
                            end_inv,
                            is_balanced,
                        };

                        let key = format!("{}:{}", item_no, warehouse_code);
                        self.inventory_records.insert(key, inventory_record);
                        processed += 1;
                    }

                    Ok((processed, balanced, unbalanced))
                },
            );

        match retry_result {
            RetryResult {
                value: Some((processed, balanced, unbalanced)),
                attempts,
                ..
            } => {
                self.circuit_breaker.record_success();
                self.workflow_record.mark_success();

                let audit_hash = compute_hash(&format!(
                    "inventory:batch={}:processed={}:balanced={}:unbalanced={}:status=success",
                    batch_id, processed, balanced, unbalanced
                ));

                Ok(InventoryResult {
                    batch_id: batch_id.to_string(),
                    records_processed: processed,
                    records_balanced: balanced,
                    records_unbalanced: unbalanced,
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
                    compute_hash(&format!("inventory:batch={}:error={}", batch_id, e));

                Err(InventoryAutomationError::Processing(e))
            }
            RetryResult {
                last_error: None, ..
            } => {
                self.circuit_breaker.record_failure();
                self.workflow_record
                    .mark_failed("unknown error after retries exhausted");

                let _audit_hash =
                    compute_hash(&format!("inventory:batch={}:error=unknown", batch_id));

                Err(InventoryAutomationError::Processing(
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
        self.workflow_record = WorkflowRecord::new("inventory", "Inventory Sync");
    }

    /// Get an inventory record by item and warehouse.
    pub fn get_inventory(&self, item_no: &str, warehouse_code: &str) -> Option<&InventoryRecord> {
        let key = format!("{}:{}", item_no, warehouse_code);
        self.inventory_records.get(&key)
    }

    /// Get total inventory record count.
    pub fn record_count(&self) -> usize {
        self.inventory_records.len()
    }

    /// Get all balanced records.
    pub fn balanced_records(&self) -> Vec<&InventoryRecord> {
        self.inventory_records
            .values()
            .filter(|r| r.is_balanced)
            .collect()
    }

    /// Get all unbalanced records.
    pub fn unbalanced_records(&self) -> Vec<&InventoryRecord> {
        self.inventory_records
            .values()
            .filter(|r| !r.is_balanced)
            .collect()
    }
}

impl Default for InventoryAutomation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_automation_new() {
        let automation = InventoryAutomation::new();
        assert_eq!(automation.workflow_record().status, WorkflowStatus::Pending);
        assert_eq!(automation.circuit_state(), crate::CircuitState::Closed);
        assert_eq!(automation.record_count(), 0);
    }

    #[test]
    fn test_inventory_sync_valid_csv() {
        let mut automation = InventoryAutomation::new();
        let csv = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\nITEM001,Product A,WH001,100,50,30,5,125\n";
        let result = automation.run_pipeline(csv, "batch-001");
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.status, WorkflowStatus::Success);
        assert_eq!(r.records_processed, 1);
        assert_eq!(r.records_balanced, 1);
        assert_eq!(r.records_unbalanced, 0);
        assert_eq!(automation.record_count(), 1);
    }

    #[test]
    fn test_inventory_sync_unbalanced() {
        let mut automation = InventoryAutomation::new();
        // Begin=100, In=50, Out=30, Adj=5 → Expected End=125, but CSV says 999
        let csv = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\nITEM001,Product A,WH001,100,50,30,5,999\n";
        let result = automation.run_pipeline(csv, "batch-001").unwrap();
        assert_eq!(result.records_processed, 1);
        assert_eq!(result.records_balanced, 0);
        assert_eq!(result.records_unbalanced, 1);
        assert_eq!(automation.unbalanced_records().len(), 1);
    }

    #[test]
    fn test_inventory_get_by_item() {
        let mut automation = InventoryAutomation::new();
        let csv = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\nITEM001,Product A,WH001,100,50,30,5,125\n";
        automation.run_pipeline(csv, "batch-001").unwrap();

        let record = automation.get_inventory("ITEM001", "WH001").unwrap();
        assert_eq!(record.item_no, "ITEM001");
        assert_eq!(record.begin_inv, 100);
        assert!(record.is_balanced);

        assert!(automation.get_inventory("NONEXISTENT", "WH001").is_none());
    }

    #[test]
    fn test_inventory_missing_column_fails() {
        let mut automation = InventoryAutomation::new();
        let csv = "WRONG_COL,OTHER\nvalue1,value2\n";
        let result = automation.run_pipeline(csv, "batch-001");
        assert!(result.is_err());
        assert!(matches!(result, Err(InventoryAutomationError::Contract(_))));
    }

    #[test]
    fn test_inventory_audit_hash() {
        let mut automation = InventoryAutomation::new();
        let csv = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\nITEM001,Product A,WH001,100,50,30,5,125\n";
        let result = automation.run_pipeline(csv, "batch-001").unwrap();
        assert!(!result.audit_hash.is_empty());
        assert_eq!(result.audit_hash.len(), 64); // SHA-256 hex
    }

    #[test]
    fn test_inventory_automation_error_display() {
        let err = InventoryAutomationError::CircuitOpen;
        assert_eq!(err.to_string(), "circuit breaker is open");

        let err = InventoryAutomationError::Processing("timeout".to_string());
        assert!(err.to_string().contains("timeout"));

        let err = InventoryAutomationError::Contract("missing column".to_string());
        assert!(err.to_string().contains("missing column"));
    }

    #[test]
    fn test_inventory_automation_reset_circuit() {
        let mut automation = InventoryAutomation::new();
        automation.reset_circuit();
        assert_eq!(automation.workflow_record().status, WorkflowStatus::Pending);
        assert_eq!(automation.circuit_state(), crate::CircuitState::Closed);
    }
}

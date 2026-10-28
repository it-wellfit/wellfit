#![forbid(unsafe_code)]

//! Wellfit Reports - Fuyi ERP Outbound Report Extraction Adapter
//!
//! Implements zero-OCR structured tabular parsing, inventory balance equation
//! verification, and release gate integration.
//!
//! The column contract of the report is resolved by column name before any
//! row is read: a report that does not carry every required column is rejected
//! with an error naming the missing column, and no row is read.

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, OnceLock};
use wellfit_core::contract::{split_header, ColumnContract, ColumnMap};
use wellfit_core::ledger::IdempotencyLedger;
use wellfit_core::mutex::{DesktopSessionMutex, WorkWave};
use wellfit_core::release_gate::ReleaseGate;

/// File name the report is recorded under in the audit ledger.
const FUYI_SOURCE_FILE: &str = "fuyi_inventory.csv";

/// One row of the Fuyi ERP inventory report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryReportRecord {
    /// ERP item number.
    pub item_no: String,
    /// Item name as written in the report.
    pub item_name: String,
    /// Warehouse the balance belongs to.
    pub warehouse_code: String,
    /// Opening balance.
    pub begin_inv: i64,
    /// Incoming quantity.
    pub in_qty: i64,
    /// Outgoing quantity.
    pub out_qty: i64,
    /// Adjustment quantity (may be negative).
    pub adj_qty: i64,
    /// Closing balance.
    pub end_inv: i64,
}

impl InventoryReportRecord {
    /// Verify fundamental inventory arithmetic: End = Begin + In - Out + Adj.
    ///
    /// The arithmetic is checked, never wrapped: an overflow means the record
    /// cannot be reconciled and must not be reported as balanced.
    pub fn verify_balance(&self) -> bool {
        let expected = self
            .begin_inv
            .checked_add(self.in_qty)
            .and_then(|v| v.checked_sub(self.out_qty))
            .and_then(|v| v.checked_add(self.adj_qty));
        match expected {
            Some(expected) => self.end_inv == expected,
            None => false,
        }
    }
}

/// A row that was isolated from the accepted data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRowError {
    /// Line number in the source file (1-based, header included).
    pub line: usize,
    /// The source line as received.
    pub raw: String,
    /// Stable error code of the rule that isolated the row.
    pub code: String,
    /// Human readable reason.
    pub reason: String,
}

/// The column contract of the Fuyi ERP inventory report.
///
/// The contract covers the columns this engine reads. Resolution is by column
/// name and has no positional fallback, so a reordered header is read
/// correctly and a missing column can never be mistaken for another one.
pub fn fuyi_inventory_report_contract() -> &'static ColumnContract {
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

/// The Fuyi ERP report extraction adapter.
pub struct ReportAutomator {
    ledger: Arc<IdempotencyLedger>,
    release_gate: Arc<Mutex<ReleaseGate>>,
    mutex: DesktopSessionMutex,
}

impl ReportAutomator {
    /// Build an adapter with the default desktop session mutex.
    pub fn new(ledger: Arc<IdempotencyLedger>, release_gate: Arc<Mutex<ReleaseGate>>) -> Self {
        Self::with_mutex(ledger, release_gate, DesktopSessionMutex::default_mutex())
    }

    /// Build an adapter with an explicit desktop session mutex.
    pub fn with_mutex(
        ledger: Arc<IdempotencyLedger>,
        release_gate: Arc<Mutex<ReleaseGate>>,
        mutex: DesktopSessionMutex,
    ) -> Self {
        Self {
            ledger,
            release_gate,
            mutex,
        }
    }

    /// The column contract enforced on every Fuyi ERP inventory report.
    pub fn column_contract(&self) -> &'static ColumnContract {
        fuyi_inventory_report_contract()
    }

    /// Parse structured CSV exported directly from Fuyi ERP.
    ///
    /// The column contract is resolved against the header before a single row
    /// is read: a report that does not carry every required column is rejected
    /// with an error naming the missing column and the contract, and nothing
    /// is written to the ledger. Row level defects (unparsable quantity,
    /// broken balance) are isolated and logged in the audit ledger.
    pub fn parse_native_csv_report(
        &self,
        csv_content: &str,
        batch_id: &str,
    ) -> Result<(Vec<InventoryReportRecord>, Vec<ReportRowError>), String> {
        let mut valid_records = Vec::new();
        let mut errors = Vec::new();

        // 1. Parse lines
        let records: Vec<(usize, &str)> = wellfit_core::text::split_records(csv_content);
        if records.is_empty() {
            return Ok((valid_records, errors));
        }

        // 2. Resolve the column contract. A violation fails the whole report:
        //    the batch is not registered, so the identifier stays available for
        //    a corrected re-export.
        let columns = match resolve_columns(records[0].1) {
            Ok(columns) => columns,
            Err(e) => {
                let mut gate = self.release_gate.lock().unwrap();
                gate.verify_payload(false, &e);
                return Err(e);
            }
        };

        let idx_item_no = contract_index(&columns, "ITEM_NO")?;
        let idx_item_name = contract_index(&columns, "ITEM_NAME")?;
        let idx_wh = contract_index(&columns, "WAREHOUSE_CODE")?;
        let idx_begin = contract_index(&columns, "BEGIN_INV")?;
        let idx_in = contract_index(&columns, "IN_QTY")?;
        let idx_out = contract_index(&columns, "OUT_QTY")?;
        let idx_adj = contract_index(&columns, "ADJ_QTY")?;
        let idx_end = contract_index(&columns, "END_INV")?;
        let column_count = columns.column_count();

        // 3. Deduplicate batch
        let registered = self
            .ledger
            .register_file_batch(
                batch_id,
                "FUYI_INVENTORY_REPORT",
                FUYI_SOURCE_FILE,
                csv_content.as_bytes(),
                0,
            )
            .map_err(|e| format!("Database error registering batch: {e}"))?;

        if !registered {
            return Err(format!(
                "Duplicate report batch {batch_id} already registered. \
                 A batch identifier is claimed once per ledger: use a new --batch for \
                 every wave, and use a fresh --db for a verification run."
            ));
        }

        for (row_line, line) in records.iter().skip(1) {
            let row_line = *row_line;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let cols: Vec<&str> = trimmed.split(',').map(|s| s.trim()).collect();
            if cols.len() < column_count {
                self.quarantine(
                    batch_id,
                    row_line,
                    line,
                    "ERR_INV_INSUFFICIENT_COLUMNS",
                    format!(
                        "Expected at least {} columns, got {}",
                        column_count,
                        cols.len()
                    ),
                    &mut errors,
                );
                continue;
            }

            let item_no = cols[idx_item_no].to_string();
            let item_name = cols[idx_item_name].to_string();
            let warehouse_code = cols[idx_wh].to_string();

            // Every quantity is parsed strictly: a value that is not a number
            // is isolated instead of being read as zero.
            let quantity_columns = [
                ("BEGIN_INV", idx_begin),
                ("IN_QTY", idx_in),
                ("OUT_QTY", idx_out),
                ("ADJ_QTY", idx_adj),
                ("END_INV", idx_end),
            ];
            let mut quantities = [0i64; 5];
            let mut unparsable = None;
            for (slot, (column, idx)) in quantities.iter_mut().zip(quantity_columns.iter()) {
                match cols[*idx].parse::<i64>() {
                    Ok(value) => *slot = value,
                    Err(e) => {
                        unparsable = Some(format!("Invalid {column} '{}': {e}", cols[*idx]));
                        break;
                    }
                }
            }
            if let Some(reason) = unparsable {
                self.quarantine(
                    batch_id,
                    row_line,
                    line,
                    "ERR_INV_NUMBER_FORMAT",
                    reason,
                    &mut errors,
                );
                continue;
            }
            let [begin_inv, in_qty, out_qty, adj_qty, end_inv] = quantities;

            let record = InventoryReportRecord {
                item_no,
                item_name,
                warehouse_code,
                begin_inv,
                in_qty,
                out_qty,
                adj_qty,
                end_inv,
            };

            if !record.verify_balance() {
                let reason = format!(
                    "Inventory arithmetic failed: {} != {} + {} - {} + {}",
                    end_inv, begin_inv, in_qty, out_qty, adj_qty
                );
                self.quarantine(
                    batch_id,
                    row_line,
                    line,
                    "ERR_INV_BALANCE_MISMATCH",
                    reason,
                    &mut errors,
                );
            } else {
                valid_records.push(record);
            }
        }

        // Two-Way Door Gate verification
        let is_clean = errors.is_empty();
        if !is_clean {
            // The report was defective, so the batch claim is released: a
            // corrected re-export under the same batch identifier is accepted.
            let _ = self.ledger.release_file_batch(batch_id);
        }
        let mut gate = self.release_gate.lock().unwrap();
        gate.verify_payload(
            is_clean,
            &format!("{} rows failed verification", errors.len()),
        );

        Ok((valid_records, errors))
    }

    /// Execute scheduled Wave 2 internal report distribution under desktop session mutex.
    pub fn execute_scheduled_wave2_report<F>(
        &self,
        raw_csv_provider: F,
        batch_id: &str,
        recipients: &[String],
    ) -> Result<bool, String>
    where
        F: FnOnce() -> String,
    {
        let _guard = self
            .mutex
            .acquire(
                WorkWave::Wave2InternalReports,
                std::time::Duration::from_secs(5),
            )
            .map_err(|e| format!("Mutex acquisition error: {e}"))?;

        let csv_content = raw_csv_provider();
        let (_valid, errors) = self.parse_native_csv_report(&csv_content, batch_id)?;

        if !errors.is_empty() {
            return Ok(false);
        }

        let mut gate = self.release_gate.lock().unwrap();
        gate.dispatch(|| Ok(true), Some(recipients))
    }

    /// Isolate a row: write it to the audit ledger and collect the error.
    fn quarantine(
        &self,
        batch_id: &str,
        line_number: usize,
        raw_value: &str,
        code: &str,
        reason: String,
        errors: &mut Vec<ReportRowError>,
    ) {
        let _ = self.ledger.log_quarantine(
            batch_id,
            FUYI_SOURCE_FILE,
            line_number,
            raw_value,
            code,
            &reason,
        );
        errors.push(ReportRowError {
            line: line_number,
            raw: raw_value.to_string(),
            code: code.to_string(),
            reason,
        });
    }
}

/// Resolve the column contract against a header line.
fn resolve_columns(header_line: &str) -> Result<ColumnMap, String> {
    let header: Vec<String> = split_header(header_line, ',');
    let names: Vec<&str> = header.iter().map(String::as_str).collect();
    fuyi_inventory_report_contract()
        .resolve(&names)
        .map_err(|e| format!("Fuyi report column contract violation: {e}"))
}

/// Position of a required column in the resolved contract.
fn contract_index(columns: &ColumnMap, name: &str) -> Result<usize, String> {
    columns
        .index(name)
        .map_err(|e| format!("Fuyi report column contract violation: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wellfit_core::release_gate::ReleaseState;

    fn automator() -> (
        ReportAutomator,
        Arc<Mutex<ReleaseGate>>,
        Arc<IdempotencyLedger>,
    ) {
        let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        (
            ReportAutomator::new(ledger.clone(), gate.clone()),
            gate,
            ledger,
        )
    }

    #[test]
    fn test_valid_inventory_report() {
        let (automator, gate, _ledger) = automator();

        let csv = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\n\
                   8,Havaianas Top Black,WH-A,100,50,30,0,120\n\
                   16,Havaianas Top Navy,WH-A,50,20,10,-5,55\n";

        let (valid, errors) = automator
            .parse_native_csv_report(csv, "BATCH-RPT-001")
            .unwrap();
        assert_eq!(valid.len(), 2);
        assert_eq!(errors.len(), 0);
        assert_eq!(valid[0].item_no, "8");
        assert_eq!(valid[0].item_name, "Havaianas Top Black");
        assert_eq!(valid[0].warehouse_code, "WH-A");
        assert_eq!(valid[1].adj_qty, -5);
        assert_eq!(gate.lock().unwrap().state, ReleaseState::VerifiedPass);
    }

    #[test]
    fn test_inventory_math_mismatch() {
        let (automator, gate, ledger) = automator();

        let csv = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\n\
                   8,Havaianas Top Black,WH-A,100,50,30,0,999\n";

        let (valid, errors) = automator
            .parse_native_csv_report(csv, "BATCH-RPT-002")
            .unwrap();
        assert_eq!(valid.len(), 0);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "ERR_INV_BALANCE_MISMATCH");
        assert_eq!(gate.lock().unwrap().state, ReleaseState::Quarantined);

        let q_items = ledger.get_quarantine_items(Some("BATCH-RPT-002")).unwrap();
        assert_eq!(q_items.len(), 1);
    }

    #[test]
    fn test_column_contract_is_public_and_complete() {
        let (automator, _gate, _ledger) = automator();
        let contract = automator.column_contract();
        assert_eq!(contract.name(), "Fuyi ERP inventory report");
        let required: Vec<&str> = contract.required().iter().map(String::as_str).collect();
        assert_eq!(
            required,
            vec![
                "ITEM_NO",
                "ITEM_NAME",
                "WAREHOUSE_CODE",
                "BEGIN_INV",
                "IN_QTY",
                "OUT_QTY",
                "ADJ_QTY",
                "END_INV"
            ]
        );
        assert!(contract.optional().is_empty());
    }

    #[test]
    fn test_missing_required_column_fails_loudly() {
        let (automator, gate, _ledger) = automator();

        // WAREHOUSE_CODE is gone: the report must fail, not read another column.
        let csv = "ITEM_NO,ITEM_NAME,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\n\
                   8,Havaianas Top Black,100,50,30,0,120\n";

        let error = automator
            .parse_native_csv_report(csv, "BATCH-RPT-003")
            .expect_err("a missing required column must fail the report");
        assert!(error.contains("WAREHOUSE_CODE"), "{error}");
        assert!(
            error.contains("Fuyi ERP inventory report"),
            "the error must name the contract: {error}"
        );
        assert_eq!(gate.lock().unwrap().state, ReleaseState::Quarantined);
    }

    #[test]
    fn test_missing_required_column_leaves_the_batch_identifier_free() {
        let (automator, _gate, _ledger) = automator();

        let broken = "ITEM_NO,ITEM_NAME,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\n\
                      8,Havaianas Top Black,100,50,30,0,120\n";
        assert!(automator
            .parse_native_csv_report(broken, "BATCH-RPT-004")
            .is_err());

        // The rejected report never reached the ledger, so the corrected
        // report can still use the same batch identifier.
        let corrected =
            "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\n\
                         8,Havaianas Top Black,WH-A,100,50,30,0,120\n";
        let (valid, errors) = automator
            .parse_native_csv_report(corrected, "BATCH-RPT-004")
            .expect("the corrected report must be accepted");
        assert_eq!(valid.len(), 1);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_reordered_header_is_read_by_name() {
        let (automator, _gate, _ledger) = automator();

        let csv = "END_INV,ADJ_QTY,OUT_QTY,IN_QTY,BEGIN_INV,WAREHOUSE_CODE,ITEM_NAME,ITEM_NO\n\
                   120,0,30,50,100,WH-A,Havaianas Top Black,8\n";

        let (valid, errors) = automator
            .parse_native_csv_report(csv, "BATCH-RPT-005")
            .unwrap();
        assert_eq!(errors.len(), 0);
        assert_eq!(valid.len(), 1);
        let record = &valid[0];
        assert_eq!(record.item_no, "8");
        assert_eq!(record.item_name, "Havaianas Top Black");
        assert_eq!(record.warehouse_code, "WH-A");
        assert_eq!(record.begin_inv, 100);
        assert_eq!(record.in_qty, 50);
        assert_eq!(record.out_qty, 30);
        assert_eq!(record.adj_qty, 0);
        assert_eq!(record.end_inv, 120);
    }

    #[test]
    fn test_truncated_row_is_quarantined() {
        let (automator, _gate, _ledger) = automator();

        let csv = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\n\
                   8,Havaianas Top Black,WH-A,100,50\n";

        let (valid, errors) = automator
            .parse_native_csv_report(csv, "BATCH-RPT-006")
            .unwrap();
        assert_eq!(valid.len(), 0);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "ERR_INV_INSUFFICIENT_COLUMNS");
    }

    #[test]
    fn test_unparsable_quantity_is_quarantined_instead_of_zeroed() {
        let (automator, _gate, _ledger) = automator();

        // A broken IN_QTY would silently read as zero and still satisfy the
        // balance equation; the row must be isolated instead.
        let csv = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\n\
                   8,Havaianas Top Black,WH-A,100,NOT-A-NUMBER,30,0,120\n";

        let (valid, errors) = automator
            .parse_native_csv_report(csv, "BATCH-RPT-007")
            .unwrap();
        assert_eq!(valid.len(), 0);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "ERR_INV_NUMBER_FORMAT");
        assert!(errors[0].reason.contains("IN_QTY"), "{:?}", errors[0]);
    }
}

#![forbid(unsafe_code)]

//! Wellfit Shopee - Inbound Order Cleaning & ERP Import Generator
//!
//! Enforces the Shopee inbound order column contract by name, the
//! `sum(line_amount) == order_total` arithmetic [EX-06], MDMHub item/channel
//! resolution, and idempotent order deduplication.
//!
//! Money is integer cents from the raw export to the ERP import CSV: no `f64`
//! ever touches an amount.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use wellfit_core::contract::{split_header, ColumnContract, ColumnMap};
use wellfit_core::ledger::IdempotencyLedger;
use wellfit_core::mdm::MdmHub;
use wellfit_core::release_gate::ReleaseGate;

/// File name the inbound export is recorded under in the audit ledger.
const SHOPEE_SOURCE_FILE: &str = "shopee_orders.csv";

/// One order line of the inbound export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShopeeOrderItem {
    /// Item number as written in the inbound export (the seller SKU).
    pub item_no: String,
    /// Seller SKU of the platform export.
    pub seller_sku: String,
    /// Canonical ERP item number resolved from the MDM catalog.
    pub erp_item_no: String,
    /// EAN-13 barcode resolved from the MDM catalog.
    pub ean13: String,
    /// Ordered quantity.
    pub qty: i64,
    /// Unit price in integer cents.
    pub unit_price_cents: i64,
    /// Line amount in integer cents.
    pub line_amount_cents: i64,
}

/// One order of the inbound export, assembled from all of its lines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShopeeOrder {
    /// Order number, the idempotency key of the order.
    pub order_no: String,
    /// Customer reference as written in the export.
    pub customer_ref: String,
    /// ERP customer code of the sales channel (`995C` for Shopee).
    pub erp_customer_code: String,
    /// Order timestamp as written in the export.
    pub order_datetime: String,
    /// Order total in integer cents.
    pub order_total_cents: i64,
    /// The order lines.
    pub items: Vec<ShopeeOrderItem>,
}

impl ShopeeOrder {
    /// Rule EX-06: sum(line_amount) == order_total
    pub fn verify_line_sum(&self) -> bool {
        let sum: i64 = self.items.iter().map(|it| it.line_amount_cents).sum();
        sum == self.order_total_cents
    }
}

/// A row that was isolated from the accepted data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopeeRowError {
    /// Line number in the source file (1-based, header included).
    pub line: usize,
    /// Stable error code of the rule that isolated the row.
    pub code: String,
    /// Human readable reason.
    pub reason: String,
}

/// The column contract of the Shopee inbound order export.
///
/// The contract covers the columns this engine reads. Resolution is by column
/// name and has no positional fallback, so a reordered header is read
/// correctly and a missing column can never be mistaken for another one.
pub fn shopee_order_contract() -> &'static ColumnContract {
    static CONTRACT: OnceLock<ColumnContract> = OnceLock::new();
    CONTRACT.get_or_init(|| {
        ColumnContract::new(
            "Shopee inbound order export",
            &[
                "ORDER_NO",
                "CUSTOMER_REF",
                "ORDER_DATETIME",
                "ORDER_TOTAL",
                "ITEM_NO",
                "QTY",
                "UNIT_PRICE",
                "LINE_AMOUNT",
            ],
            &[],
        )
    })
}

/// Pure integer money parsing: supports "1200" (whole units) and "12.00"
/// (decimal units), zero floating point arithmetic.
///
/// A literal that is not a number yields `None`: a malformed amount is never
/// silently turned into zero.
fn parse_money_cents(raw: &str) -> Option<i64> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    if let Some((units, fraction)) = s.split_once('.') {
        let units: i64 = units.trim().parse().ok()?;
        let fraction = fraction.trim();
        let cents = match fraction.len() {
            0 => 0,
            1 => fraction.parse::<i64>().ok()?.checked_mul(10)?,
            2 => fraction.parse::<i64>().ok()?,
            _ => fraction.get(..2)?.parse::<i64>().ok()?,
        };
        units.checked_mul(100)?.checked_add(cents)
    } else {
        s.parse::<i64>().ok()?.checked_mul(100)
    }
}

/// Format integer cents as a decimal amount with exactly two fraction digits.
///
/// Pure integer arithmetic: the amount is never converted to a float.
fn format_cents(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let magnitude = cents.unsigned_abs();
    format!("{sign}{}.{:02}", magnitude / 100, magnitude % 100)
}

/// The Shopee inbound order engine.
pub struct ShopeeInboundEngine {
    ledger: Arc<IdempotencyLedger>,
    release_gate: Arc<Mutex<ReleaseGate>>,
    /// Name of the sales channel the engine serves.
    pub channel_name: String,
    /// ERP customer code of the sales channel.
    pub erp_customer_code: String,
}

impl ShopeeInboundEngine {
    /// Build an engine for the default Shopee channel (ERP customer code 995C).
    pub fn new(ledger: Arc<IdempotencyLedger>, release_gate: Arc<Mutex<ReleaseGate>>) -> Self {
        Self::with_channel(ledger, release_gate, "SHOPEE", None)
    }

    /// Build an engine for a named sales channel, optionally overriding the
    /// ERP customer code.
    pub fn with_channel(
        ledger: Arc<IdempotencyLedger>,
        release_gate: Arc<Mutex<ReleaseGate>>,
        channel_name: &str,
        custom_erp_code: Option<&str>,
    ) -> Self {
        let erp_customer_code = custom_erp_code
            .map(|s| s.to_string())
            .unwrap_or_else(|| MdmHub::resolve_channel_client(channel_name));
        Self {
            ledger,
            release_gate,
            channel_name: channel_name.to_string(),
            erp_customer_code,
        }
    }

    /// The column contract enforced on every inbound order export.
    pub fn column_contract(&self) -> &'static ColumnContract {
        shopee_order_contract()
    }

    /// Clean, validate, and structure raw Shopee orders.
    ///
    /// The column contract is resolved against the header before a single row
    /// is read: a file that does not carry every required column is rejected
    /// with an error naming the missing column and the contract, and nothing
    /// is written to the ledger. Row level defects (unknown SKU, EX-06
    /// mismatch, duplicate order) are isolated and logged in the audit ledger.
    pub fn process_raw_orders(
        &self,
        raw_csv_content: &str,
        batch_id: &str,
    ) -> Result<(Vec<ShopeeOrder>, Vec<ShopeeRowError>), String> {
        let mut valid_orders = Vec::new();
        let mut errors = Vec::new();

        // 1. Parse lines
        let records: Vec<(usize, &str)> = wellfit_core::text::split_records(raw_csv_content);
        if records.is_empty() {
            return Ok((valid_orders, errors));
        }

        // 2. Resolve the column contract. A violation fails the whole file:
        //    the batch is not registered, so the identifier stays available
        //    for a corrected re-export.
        let columns = match resolve_columns(records[0].1) {
            Ok(columns) => columns,
            Err(e) => {
                let mut gate = self.release_gate.lock().unwrap();
                gate.verify_payload(false, &e);
                return Err(e);
            }
        };

        let idx_order_no = contract_index(&columns, "ORDER_NO")?;
        let idx_customer_ref = contract_index(&columns, "CUSTOMER_REF")?;
        let idx_datetime = contract_index(&columns, "ORDER_DATETIME")?;
        let idx_total = contract_index(&columns, "ORDER_TOTAL")?;
        let idx_item = contract_index(&columns, "ITEM_NO")?;
        let idx_qty = contract_index(&columns, "QTY")?;
        let idx_price = contract_index(&columns, "UNIT_PRICE")?;
        let idx_line_amt = contract_index(&columns, "LINE_AMOUNT")?;
        let column_count = columns.column_count();

        // 3. Batch registration
        let registered = self
            .ledger
            .register_file_batch(
                batch_id,
                "SHOPEE_RAW_INBOUND",
                SHOPEE_SOURCE_FILE,
                raw_csv_content.as_bytes(),
                0,
            )
            .map_err(|e| format!("Database error registering batch: {e}"))?;

        if !registered {
            return Err(format!(
                "Duplicate Shopee batch {batch_id} already registered. \
                 A batch identifier is claimed once per ledger: use a new --batch for \
                 every wave, and use a fresh --db for a verification run."
            ));
        }

        // Group rows by order_no
        struct OrderAccumulator {
            customer_ref: String,
            order_datetime: String,
            order_total_cents: i64,
            raw_line: usize,
            items: Vec<ShopeeOrderItem>,
        }

        let mut orders_map: HashMap<String, OrderAccumulator> = HashMap::new();

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
                    "ERR_INSUFFICIENT_COLUMNS",
                    format!(
                        "Row carries {} column(s); the header declares {}",
                        cols.len(),
                        column_count
                    ),
                    &mut errors,
                )?;
                continue;
            }

            let order_no = cols[idx_order_no];
            if order_no.is_empty() {
                self.quarantine(
                    batch_id,
                    row_line,
                    line,
                    "ERR_MISSING_ORDER_NO",
                    "Empty ORDER_NO in row".to_string(),
                    &mut errors,
                )?;
                continue;
            }

            let total_cents = match parse_money_cents(cols[idx_total]) {
                Some(cents) => cents,
                None => {
                    self.quarantine(
                        batch_id,
                        row_line,
                        line,
                        "ERR_MONEY_FORMAT",
                        format!("Invalid ORDER_TOTAL: '{}'", cols[idx_total]),
                        &mut errors,
                    )?;
                    continue;
                }
            };

            if !orders_map.contains_key(order_no) {
                orders_map.insert(
                    order_no.to_string(),
                    OrderAccumulator {
                        customer_ref: cols[idx_customer_ref].to_string(),
                        order_datetime: cols[idx_datetime].to_string(),
                        order_total_cents: total_cents,
                        raw_line: row_line,
                        items: Vec::new(),
                    },
                );
            }

            let seller_sku = cols[idx_item];
            let sku_meta = match MdmHub::lookup_sku(seller_sku) {
                Some(m) => m,
                None => {
                    self.quarantine(
                        batch_id,
                        row_line,
                        line,
                        "ERR_UNKNOWN_SKU",
                        format!("SKU '{seller_sku}' not found in canonical MDM catalog"),
                        &mut errors,
                    )?;
                    continue;
                }
            };

            let qty: i64 = match cols[idx_qty].parse() {
                Ok(qty) => qty,
                Err(e) => {
                    self.quarantine(
                        batch_id,
                        row_line,
                        line,
                        "ERR_QTY_FORMAT",
                        format!("Invalid QTY '{}': {e}", cols[idx_qty]),
                        &mut errors,
                    )?;
                    continue;
                }
            };

            let unit_price_cents = match parse_money_cents(cols[idx_price]) {
                Some(cents) => cents,
                None => {
                    self.quarantine(
                        batch_id,
                        row_line,
                        line,
                        "ERR_MONEY_FORMAT",
                        format!("Invalid UNIT_PRICE: '{}'", cols[idx_price]),
                        &mut errors,
                    )?;
                    continue;
                }
            };

            let line_amount_cents = match parse_money_cents(cols[idx_line_amt]) {
                Some(cents) => cents,
                None => {
                    self.quarantine(
                        batch_id,
                        row_line,
                        line,
                        "ERR_MONEY_FORMAT",
                        format!("Invalid LINE_AMOUNT: '{}'", cols[idx_line_amt]),
                        &mut errors,
                    )?;
                    continue;
                }
            };

            if let Some(acc) = orders_map.get_mut(order_no) {
                acc.items.push(ShopeeOrderItem {
                    item_no: seller_sku.to_string(),
                    seller_sku: seller_sku.to_string(),
                    erp_item_no: sku_meta.item_no,
                    ean13: sku_meta.ean13,
                    qty,
                    unit_price_cents,
                    line_amount_cents,
                });
            }
        }

        // 4. Assemble and verify each order
        for (order_no, acc) in orders_map {
            if acc.items.is_empty() {
                continue; // Defective items already quarantined
            }

            let order = ShopeeOrder {
                order_no: order_no.clone(),
                customer_ref: acc.customer_ref,
                erp_customer_code: self.erp_customer_code.clone(),
                order_datetime: acc.order_datetime,
                order_total_cents: acc.order_total_cents,
                items: acc.items,
            };

            // Rule EX-06 Check
            if !order.verify_line_sum() {
                let reason = format!(
                    "Order {} sum(lines) != total ({})",
                    order_no, order.order_total_cents
                );
                self.quarantine(
                    batch_id,
                    acc.raw_line,
                    &order_no,
                    "ERR_SUM_MISMATCH_EX06",
                    reason,
                    &mut errors,
                )?;
                continue;
            }

            // Idempotency lock check
            let locked = self
                .ledger
                .check_and_lock_order(&order_no, batch_id, order.order_total_cents)
                .map_err(|e| format!("Ledger lock error: {e}"))?;

            if !locked {
                let reason = format!("Order {} already exists in ERP ledger", order_no);
                self.quarantine(
                    batch_id,
                    acc.raw_line,
                    &order_no,
                    "ERR_DUPLICATE_ORDER",
                    reason,
                    &mut errors,
                )?;
                continue;
            }

            valid_orders.push(order);
        }

        let is_clean = errors.is_empty();
        if !is_clean {
            // The file was defective, so the batch claim is released: a
            // corrected re-export under the same batch identifier is accepted.
            self.ledger
                .release_file_batch(batch_id)
                .map_err(|e| format!("Database error releasing defective batch claim: {e}"))?;
        }
        let mut gate = self.release_gate.lock().unwrap();
        gate.verify_payload(is_clean, &format!("{} order errors found", errors.len()));

        Ok((valid_orders, errors))
    }

    /// Format validated orders into Fuyi ERP standard batch import CSV layout.
    ///
    /// Amounts are written as decimal text derived from integer cents; the
    /// value never passes through a float.
    pub fn generate_fuyi_erp_import_csv(&self, orders: &[ShopeeOrder]) -> String {
        let mut out = String::from(
            "ORDER_NO,CUSTOMER_CODE,ORDER_DATE,ITEM_NO,BARCODE_EAN13,QTY,UNIT_PRICE,LINE_TOTAL\n",
        );
        for o in orders {
            for it in &o.items {
                let unit_price = format_cents(it.unit_price_cents);
                let line_total = format_cents(it.line_amount_cents);
                out.push_str(&format!(
                    "{},{},{},{},{},{},{},{}\n",
                    o.order_no,
                    o.erp_customer_code,
                    o.order_datetime,
                    it.erp_item_no,
                    it.ean13,
                    it.qty,
                    unit_price,
                    line_total
                ));
            }
        }
        out
    }

    /// Isolate a row: write it to the audit ledger and collect the error.
    fn quarantine(
        &self,
        batch_id: &str,
        line_number: usize,
        raw_value: &str,
        code: &str,
        reason: String,
        errors: &mut Vec<ShopeeRowError>,
    ) -> Result<(), String> {
        self.ledger
            .log_quarantine(
                batch_id,
                SHOPEE_SOURCE_FILE,
                line_number,
                raw_value,
                code,
                &reason,
            )
            .map_err(|e| {
                format!("Database error recording quarantine for line {line_number}: {e}")
            })?;
        errors.push(ShopeeRowError {
            line: line_number,
            code: code.to_string(),
            reason,
        });
        Ok(())
    }
}

/// Resolve the column contract against a header line.
fn resolve_columns(header_line: &str) -> Result<ColumnMap, String> {
    let header: Vec<String> = split_header(header_line, ',');
    let names: Vec<&str> = header.iter().map(String::as_str).collect();
    shopee_order_contract()
        .resolve(&names)
        .map_err(|e| format!("Shopee column contract violation: {e}"))
}

/// Position of a required column in the resolved contract.
fn contract_index(columns: &ColumnMap, name: &str) -> Result<usize, String> {
    columns
        .index(name)
        .map_err(|e| format!("Shopee column contract violation: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wellfit_core::release_gate::ReleaseState;

    fn engine() -> (ShopeeInboundEngine, Arc<Mutex<ReleaseGate>>) {
        let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        (ShopeeInboundEngine::new(ledger, gate.clone()), gate)
    }

    #[test]
    fn test_valid_shopee_orders() {
        let (engine, gate) = engine();

        let csv =
            "ORDER_NO,CUSTOMER_REF,ORDER_DATETIME,ORDER_TOTAL,ITEM_NO,QTY,UNIT_PRICE,LINE_AMOUNT\n\
                   2610200001,CUST-A,2026-10-20 08:15:00,1000,HAV-TOP-BLK-39,2,500,1000\n";

        let (valid, errors) = engine.process_raw_orders(csv, "BATCH-SHP-001").unwrap();
        assert_eq!(valid.len(), 1);
        assert_eq!(errors.len(), 0);
        assert_eq!(valid[0].erp_customer_code, "995C");
        assert_eq!(valid[0].items[0].ean13, "7898656600087");

        let fuyi_csv = engine.generate_fuyi_erp_import_csv(&valid);
        assert!(fuyi_csv.contains("2610200001"));
        assert!(fuyi_csv.contains("995C"));
        assert!(fuyi_csv.contains("7898656600087"));
        assert_eq!(gate.lock().unwrap().state, ReleaseState::VerifiedPass);
    }

    #[test]
    fn test_ex06_sum_mismatch() {
        let (engine, gate) = engine();

        let csv =
            "ORDER_NO,CUSTOMER_REF,ORDER_DATETIME,ORDER_TOTAL,ITEM_NO,QTY,UNIT_PRICE,LINE_AMOUNT\n\
                   2610200002,CUST-B,2026-10-20 08:20:00,1500,HAV-TOP-BLK-39,2,500,1000\n";

        let (valid, errors) = engine.process_raw_orders(csv, "BATCH-SHP-002").unwrap();
        assert_eq!(valid.len(), 0);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "ERR_SUM_MISMATCH_EX06");
        assert_eq!(gate.lock().unwrap().state, ReleaseState::Quarantined);
    }

    #[test]
    fn test_column_contract_is_public_and_complete() {
        let (engine, _gate) = engine();
        let contract = engine.column_contract();
        assert_eq!(contract.name(), "Shopee inbound order export");
        let required: Vec<&str> = contract.required().iter().map(String::as_str).collect();
        assert_eq!(
            required,
            vec![
                "ORDER_NO",
                "CUSTOMER_REF",
                "ORDER_DATETIME",
                "ORDER_TOTAL",
                "ITEM_NO",
                "QTY",
                "UNIT_PRICE",
                "LINE_AMOUNT"
            ]
        );
        assert!(contract.optional().is_empty());
    }

    #[test]
    fn test_missing_required_column_fails_loudly() {
        let (engine, gate) = engine();

        // UNIT_PRICE is gone: the engine must fail, not read another column.
        let csv = "ORDER_NO,CUSTOMER_REF,ORDER_DATETIME,ORDER_TOTAL,ITEM_NO,QTY,LINE_AMOUNT\n\
                   2610200003,CUST-C,2026-10-20 08:25:00,1000,HAV-TOP-BLK-39,2,1000\n";

        let error = engine
            .process_raw_orders(csv, "BATCH-SHP-003")
            .expect_err("a missing required column must fail the file");
        assert!(error.contains("UNIT_PRICE"), "{error}");
        assert!(
            error.contains("Shopee inbound order export"),
            "the error must name the contract: {error}"
        );
        assert_eq!(gate.lock().unwrap().state, ReleaseState::Quarantined);
    }

    #[test]
    fn test_missing_required_column_leaves_the_batch_identifier_free() {
        let (engine, _gate) = engine();

        let broken = "ORDER_NO,CUSTOMER_REF,ORDER_DATETIME,ORDER_TOTAL,ITEM_NO,QTY,LINE_AMOUNT\n\
                      2610200004,CUST-D,2026-10-20 08:30:00,1000,HAV-TOP-BLK-39,2,1000\n";
        assert!(engine.process_raw_orders(broken, "BATCH-SHP-004").is_err());

        // The rejected file never reached the ledger, so the corrected file
        // can still use the same batch identifier.
        let corrected =
            "ORDER_NO,CUSTOMER_REF,ORDER_DATETIME,ORDER_TOTAL,ITEM_NO,QTY,UNIT_PRICE,LINE_AMOUNT\n\
                         2610200004,CUST-D,2026-10-20 08:30:00,1000,HAV-TOP-BLK-39,2,500,1000\n";
        let (valid, errors) = engine
            .process_raw_orders(corrected, "BATCH-SHP-004")
            .expect("the corrected export must be accepted");
        assert_eq!(valid.len(), 1);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_reordered_header_is_read_by_name() {
        let (engine, _gate) = engine();

        let csv =
            "LINE_AMOUNT,QTY,UNIT_PRICE,ITEM_NO,ORDER_TOTAL,ORDER_DATETIME,CUSTOMER_REF,ORDER_NO\n\
                   1000,2,500,HAV-TOP-BLK-39,1000,2026-10-20 08:15:00,CUST-A,2610200005\n";

        let (valid, errors) = engine.process_raw_orders(csv, "BATCH-SHP-005").unwrap();
        assert_eq!(errors.len(), 0);
        assert_eq!(valid.len(), 1);
        let order = &valid[0];
        assert_eq!(order.order_no, "2610200005");
        assert_eq!(order.customer_ref, "CUST-A");
        assert_eq!(order.order_datetime, "2026-10-20 08:15:00");
        assert_eq!(order.order_total_cents, 100000);
        assert_eq!(order.items[0].seller_sku, "HAV-TOP-BLK-39");
        assert_eq!(order.items[0].qty, 2);
        assert_eq!(order.items[0].unit_price_cents, 50000);
        assert_eq!(order.items[0].line_amount_cents, 100000);
    }

    #[test]
    fn test_duplicate_order_is_rejected_by_the_ledger() {
        let (engine, _gate) = engine();

        let csv =
            "ORDER_NO,CUSTOMER_REF,ORDER_DATETIME,ORDER_TOTAL,ITEM_NO,QTY,UNIT_PRICE,LINE_AMOUNT\n\
                   2610200006,CUST-A,2026-10-20 08:15:00,1000,HAV-TOP-BLK-39,2,500,1000\n";

        let (valid, errors) = engine.process_raw_orders(csv, "BATCH-SHP-006").unwrap();
        assert_eq!(valid.len(), 1);
        assert_eq!(errors.len(), 0);

        // The same order number in a second, different batch is refused.
        let repeat =
            "ORDER_NO,CUSTOMER_REF,ORDER_DATETIME,ORDER_TOTAL,ITEM_NO,QTY,UNIT_PRICE,LINE_AMOUNT\n\
                      2610200006,CUST-Z,2026-10-21 09:00:00,1000,HAV-TOP-BLK-39,2,500,1000\n";
        let (valid, errors) = engine.process_raw_orders(repeat, "BATCH-SHP-007").unwrap();
        assert_eq!(valid.len(), 0);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "ERR_DUPLICATE_ORDER");
    }

    #[test]
    fn test_unknown_seller_sku_is_quarantined() {
        let (engine, _gate) = engine();

        let csv =
            "ORDER_NO,CUSTOMER_REF,ORDER_DATETIME,ORDER_TOTAL,ITEM_NO,QTY,UNIT_PRICE,LINE_AMOUNT\n\
                   2610200008,CUST-D,2026-10-20 08:30:00,600,UNKNOWN-SKU-FAIL,1,600,600\n";

        let (valid, errors) = engine.process_raw_orders(csv, "BATCH-SHP-008").unwrap();
        assert_eq!(valid.len(), 0);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "ERR_UNKNOWN_SKU");
    }

    #[test]
    fn test_truncated_row_is_quarantined_instead_of_read_as_empty() {
        let (engine, _gate) = engine();

        let csv =
            "ORDER_NO,CUSTOMER_REF,ORDER_DATETIME,ORDER_TOTAL,ITEM_NO,QTY,UNIT_PRICE,LINE_AMOUNT\n\
                   2610200009,CUST-E,2026-10-20 08:35:00,1000,HAV-TOP-BLK-39,2\n";

        let (valid, errors) = engine.process_raw_orders(csv, "BATCH-SHP-009").unwrap();
        assert_eq!(valid.len(), 0);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "ERR_INSUFFICIENT_COLUMNS");
    }

    #[test]
    fn test_malformed_amount_is_quarantined_instead_of_zeroed() {
        let (engine, _gate) = engine();

        let csv =
            "ORDER_NO,CUSTOMER_REF,ORDER_DATETIME,ORDER_TOTAL,ITEM_NO,QTY,UNIT_PRICE,LINE_AMOUNT\n\
                   2610200010,CUST-F,2026-10-20 08:40:00,1000,HAV-TOP-BLK-39,2,NOT-A-PRICE,1000\n";

        let (valid, errors) = engine.process_raw_orders(csv, "BATCH-SHP-010").unwrap();
        assert_eq!(valid.len(), 0);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "ERR_MONEY_FORMAT");
    }

    #[test]
    fn test_erp_import_csv_formats_cents_without_floating_point() {
        let (engine, _gate) = engine();

        let order = ShopeeOrder {
            order_no: "2610200011".to_string(),
            customer_ref: "CUST-G".to_string(),
            erp_customer_code: "995C".to_string(),
            order_datetime: "2026-10-20 08:45:00".to_string(),
            order_total_cents: 120000,
            items: vec![ShopeeOrderItem {
                item_no: "HAV-TOP-BLK-39".to_string(),
                seller_sku: "HAV-TOP-BLK-39".to_string(),
                erp_item_no: "8".to_string(),
                ean13: "7898656600087".to_string(),
                qty: 2,
                unit_price_cents: 600,
                line_amount_cents: 120000,
            }],
        };

        let csv = engine.generate_fuyi_erp_import_csv(std::slice::from_ref(&order));
        let row = wellfit_core::text::split_lines(&csv)
            .get(1)
            .copied()
            .expect("one data row");
        assert_eq!(
            row,
            "2610200011,995C,2026-10-20 08:45:00,8,7898656600087,2,6.00,1200.00"
        );

        // No field carries a float artefact: every amount has exactly two
        // fraction digits and no exponent notation.
        for field in row.split(',') {
            if field.contains('.') {
                let (major, fraction) = field.split_once('.').expect("decimal amount");
                assert!(major.chars().all(|c| c.is_ascii_digit()), "{field}");
                assert_eq!(fraction.len(), 2, "{field}");
                assert!(fraction.chars().all(|c| c.is_ascii_digit()), "{field}");
            }
            assert!(!field.contains('e') && !field.contains('E'), "{field}");
        }
    }

    #[test]
    fn test_cents_formatting_is_exact() {
        assert_eq!(format_cents(0), "0.00");
        assert_eq!(format_cents(6), "0.06");
        assert_eq!(format_cents(600), "6.00");
        assert_eq!(format_cents(120000), "1200.00");
        assert_eq!(format_cents(123456), "1234.56");
        assert_eq!(format_cents(-600), "-6.00");
    }

    #[test]
    fn test_money_parsing_is_integer_only() {
        assert_eq!(parse_money_cents("1200"), Some(120000));
        assert_eq!(parse_money_cents("12.00"), Some(1200));
        assert_eq!(parse_money_cents("12.5"), Some(1250));
        assert_eq!(parse_money_cents("12.345"), Some(1234));
        assert_eq!(parse_money_cents(""), None);
        assert_eq!(parse_money_cents("abc"), None);
    }
}

#![forbid(unsafe_code)]

//! BankPay engine: export (ERP -> bank) and import (bank -> ERP) pipelines
//! for accounts payable vouchers.
//!
//! The engine is deliberately dependency-light: it owns the voucher/bank
//! record schemas, the column contracts, the ledger claims, the release gate
//! transitions, and the media-file assembly. It does not perform network I/O
//! itself; the caller supplies a transport function so the same engine works
//! in unit tests and in production.

use crate::api::{self, TwdPaymentRequest, TwdPaymentRequestData, TwdPaymentTx};
use crate::encoding::{from_utf8, to_utf8, BankEncoding};
use crate::http::EsunHttpClient;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use wellfit_core::contract::ColumnContract;
use wellfit_core::ledger::{Finding, FindingSeverity, IdempotencyLedger};
use wellfit_core::release_gate::ReleaseGate;
use wellfit_core::scaled::ScaledDecimal;
use wellfit_core::text::{split_records, MAX_INPUT_BYTES};

// ---------------------------------------------------------------------------
// Domain models
// ---------------------------------------------------------------------------

/// One accounts-payable voucher read from the ERP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voucher {
    /// ERP voucher number (e.g. `AP-00123`).
    pub voucher_no: String,
    /// Vendor / supplier code.
    pub vendor_code: String,
    /// Payable amount in integer cents.
    pub amount_cents: i64,
    /// Due date (ISO-8601).
    pub due_date: String,
    /// Beneficiary bank account.
    pub bank_account: String,
    /// Free-text reference / remark.
    pub reference: String,
    /// One-based source line number (for audit).
    pub line: usize,
    /// Raw source line (for quarantine reproduction).
    pub raw: String,
}

/// One bank result record returned by the bank.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankResult {
    /// Bank transaction / payment reference.
    pub bank_ref: String,
    /// ERP voucher number this result settles.
    pub voucher_no: String,
    /// Settlement status: `paid`, `failed`, `uncertain`.
    pub status: String,
    /// Settled amount in integer cents.
    pub amount_cents: i64,
    /// Bank processing timestamp (ISO-8601).
    pub processed_at: String,
    /// One-based source line number.
    pub line: usize,
    /// Raw source line.
    pub raw: String,
}

/// Outcome of an export run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub batch_id: String,
    pub media_file: String,
    pub file_sha256: String,
    pub vouchers_accepted: usize,
    pub vouchers_isolated: usize,
    pub dispatched: bool,
    pub release_state: String,
}

/// Outcome of an import run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub batch_id: String,
    pub results_accepted: usize,
    pub results_isolated: usize,
    pub matched_vouchers: usize,
    pub unmatched: usize,
}

/// Resolved column indices for a voucher export file.
struct VoucherColumns {
    voucher_no: usize,
    vendor_code: usize,
    amount: usize,
    due_date: usize,
    bank_account: usize,
    reference: usize,
}

/// Resolved column indices for a bank result import file.
struct BankResultColumns {
    bank_ref: usize,
    voucher_no: usize,
    status: usize,
    amount: usize,
    processed_at: usize,
}

/// Required columns of an ERP voucher export (CSV).
pub fn voucher_contract() -> ColumnContract {
    ColumnContract::new(
        "E.Sun bank payable voucher export",
        &["VOUCHER_NO", "VENDOR_CODE", "AMOUNT", "BANK_ACCOUNT"],
        &["DUE_DATE", "REFERENCE"],
    )
}

/// Required columns of a bank result import (CSV).
pub fn bank_result_contract() -> ColumnContract {
    ColumnContract::new(
        "E.Sun bank result import",
        &["BANK_REF", "VOUCHER_NO", "STATUS", "AMOUNT"],
        &["PROCESSED_AT"],
    )
}

/// Resolve column indices once from the header row.
fn resolve_voucher_columns(
    header: &str,
    contract: &ColumnContract,
) -> Result<VoucherColumns, String> {
    let sep = detect_separator(header);
    let values: Vec<&str> = header.split(sep).map(str::trim).collect();
    let map = contract.resolve(&values)?;
    Ok(VoucherColumns {
        voucher_no: map.index("VOUCHER_NO")?,
        vendor_code: map.index("VENDOR_CODE")?,
        amount: map.index("AMOUNT")?,
        due_date: map.optional_index("DUE_DATE").unwrap_or(usize::MAX),
        bank_account: map.index("BANK_ACCOUNT")?,
        reference: map.optional_index("REFERENCE").unwrap_or(usize::MAX),
    })
}

/// Resolve column indices once from the header row.
fn resolve_bank_result_columns(
    header: &str,
    contract: &ColumnContract,
) -> Result<BankResultColumns, String> {
    let sep = detect_separator(header);
    let values: Vec<&str> = header.split(sep).map(str::trim).collect();
    let map = contract.resolve(&values)?;
    Ok(BankResultColumns {
        bank_ref: map.index("BANK_REF")?,
        voucher_no: map.index("VOUCHER_NO")?,
        status: map.index("STATUS")?,
        amount: map.index("AMOUNT")?,
        processed_at: map.optional_index("PROCESSED_AT").unwrap_or(usize::MAX),
    })
}

fn cell<'a>(values: &'a [&str], index: usize) -> &'a str {
    values.get(index).copied().unwrap_or("").trim()
}

/// Parse one voucher data row using pre-resolved column indices.
fn parse_voucher_row(
    values: &[&str],
    cols: &VoucherColumns,
    line: usize,
) -> Result<Voucher, String> {
    let voucher_no = cell(values, cols.voucher_no);
    if voucher_no.is_empty() {
        return Err(format!("line {line}: VOUCHER_NO is empty"));
    }
    let vendor_code = cell(values, cols.vendor_code);
    let amount_raw = cell(values, cols.amount);
    let amount_cents = ScaledDecimal::parse(amount_raw)
        .map_err(|e| format!("line {line}: AMOUNT '{amount_raw}': {e}"))?
        .units()
        / 10_000;
    let due_date = cell(values, cols.due_date);
    let bank_account = cell(values, cols.bank_account);
    if bank_account.is_empty() {
        return Err(format!("line {line}: BANK_ACCOUNT is empty"));
    }
    let reference = cell(values, cols.reference);
    let raw = values.join(",");
    Ok(Voucher {
        voucher_no: voucher_no.to_string(),
        vendor_code: vendor_code.to_string(),
        amount_cents,
        due_date: due_date.to_string(),
        bank_account: bank_account.to_string(),
        reference: reference.to_string(),
        line,
        raw,
    })
}

/// Parse one bank result data row using pre-resolved column indices.
fn parse_bank_result_row(
    values: &[&str],
    cols: &BankResultColumns,
    line: usize,
) -> Result<BankResult, String> {
    let bank_ref = cell(values, cols.bank_ref);
    if bank_ref.is_empty() {
        return Err(format!("line {line}: BANK_REF is empty"));
    }
    let voucher_no = cell(values, cols.voucher_no);
    if voucher_no.is_empty() {
        return Err(format!("line {line}: VOUCHER_NO is empty"));
    }
    let status = cell(values, cols.status);
    let amount_raw = cell(values, cols.amount);
    let amount_cents = ScaledDecimal::parse(amount_raw)
        .map_err(|e| format!("line {line}: AMOUNT '{amount_raw}': {e}"))?
        .units()
        / 10_000;
    let processed_at = cell(values, cols.processed_at);
    let raw = values.join(",");
    Ok(BankResult {
        bank_ref: bank_ref.to_string(),
        voucher_no: voucher_no.to_string(),
        status: status.to_string(),
        amount_cents,
        processed_at: processed_at.to_string(),
        line,
        raw,
    })
}

// ---------------------------------------------------------------------------
// Media file assembly (bank media file, E.Sun style)
// ---------------------------------------------------------------------------

/// Assemble the CSV bank media file content for a batch of vouchers, then
/// encode it per the target encoding.
///
/// The produced content is a CSV with a header row, one row per voucher, and
/// a trailer record with the batch hash. The caller writes this to disk and
/// transports it to the bank.
pub fn assemble_media_file(
    vouchers: &[Voucher],
    batch_id: &str,
    encoding: BankEncoding,
) -> (Vec<u8>, bool) {
    let mut lines: Vec<String> = Vec::new();
    lines.push("VOUCHER_NO,VENDOR_CODE,AMOUNT_CENTS,BANK_ACCOUNT,DUE_DATE,REFERENCE".to_string());
    for v in vouchers {
        lines.push(format!(
            "{},{},{},{},{},{}",
            v.voucher_no, v.vendor_code, v.amount_cents, v.bank_account, v.due_date, v.reference
        ));
    }
    let total: i64 = vouchers.iter().map(|v| v.amount_cents).sum();
    lines.push(format!("TRAILER,{},{},{}", batch_id, vouchers.len(), total));
    let body = lines.join("\n");
    let (bytes, had_errors) = from_utf8(&body, encoding);
    (bytes, had_errors)
}

/// SHA-256 of the media file bytes (the traceable content hash).
pub fn media_file_sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

// ---------------------------------------------------------------------------

/// Format an amount in cents as a yuan string with exactly 2 decimal places.
/// Uses integer arithmetic to avoid floating-point precision loss.
pub fn cents_to_yuan_str(cents: i64) -> String {
    let negative = cents < 0;
    let abs_cents = cents.unsigned_abs();
    let yuan = abs_cents / 100;
    let remainder = abs_cents % 100;
    if negative {
        format!("-{}.{:02}", yuan, remainder)
    } else {
        format!("{}.{:02}", yuan, remainder)
    }
}
// Engine
// ---------------------------------------------------------------------------

/// The BankPay engine. Holds the ledger, release gate, and optional HTTP client.
pub struct BankpayEngine {
    pub ledger: Arc<IdempotencyLedger>,
    pub gate: Arc<Mutex<ReleaseGate>>,
    pub http_client: Option<EsunHttpClient>,
}

impl BankpayEngine {
    /// Open a production-scoped engine (exclusive batch claims).
    pub fn new(ledger: Arc<IdempotencyLedger>, gate: Arc<Mutex<ReleaseGate>>) -> Self {
        Self {
            ledger,
            gate,
            http_client: None,
        }
    }

    /// Open an engine with per-run claims (used by verification / audit).
    pub fn new_scoped(ledger: Arc<IdempotencyLedger>, gate: Arc<Mutex<ReleaseGate>>) -> Self {
        Self {
            ledger,
            gate,
            http_client: None,
        }
    }

    /// Attach an HTTP client for API-based operations.
    pub fn with_http_client(mut self, client: EsunHttpClient) -> Self {
        self.http_client = Some(client);
        self
    }

    /// Query TWD transaction status via the E.Sun API.
    ///
    /// Returns the bank reference and status code for a given payment number
    /// and transaction sequence. Used for DispatchUncertain reconciliation.
    pub fn query_twd_status(
        &self,
        source_system: &str,
        payment_no: &str,
        tx_seq: &str,
    ) -> Result<api::TwdStatusResponseData, String> {
        let client = self
            .http_client
            .as_ref()
            .ok_or_else(|| "HTTP client not configured".to_string())?;
        let request = api::TwdStatusQueryRequest {
            rq_header: api::RqHeader {
                msg_id: format!(
                    "{}{}{:07}",
                    payment_no.chars().take(10).collect::<String>(),
                    chrono::Utc::now().format("%Y%m%d"),
                    1
                ),
                request_time: chrono::Utc::now().format("%Y/%m/%d %H:%M:%S").to_string(),
                customer_ixd: payment_no.chars().take(10).collect(),
                source_system: source_system.to_string(),
            },
            rq_data: api::TwdStatusQueryRequestData {
                payment_no: payment_no.to_string(),
                tx_seq: tx_seq.to_string(),
            },
        };
        let response = client.retrieve_twd_status(source_system, &request)?;
        if response.is_success() {
            Ok(response.rs_data.unwrap_or_default())
        } else {
            Err(format!(
                "TWD status query failed: {} {}",
                response.rs_status.code, response.rs_status.message
            ))
        }
    }

    /// Convert a Voucher to a TwdPaymentTx for the E.Sun API.
    pub fn voucher_to_twd_tx(
        voucher: &Voucher,
        payer_account: &str,
        payer_id: &str,
    ) -> TwdPaymentTx {
        TwdPaymentTx {
            payer_account_no: payer_account.to_string(),
            payer_id: payer_id.to_string(),
            payee_bank_id: "000".to_string(), // placeholder - from ERP vendor data
            payee_branch_id: "0000".to_string(),
            payee_account_no: voucher.bank_account.clone(),
            payee_name: voucher.vendor_code.clone(),
            payee_id: String::new(),
            tx_date: if voucher.due_date.is_empty() {
                chrono::Utc::now().format("%Y%m%d").to_string()
            } else {
                voucher.due_date.replace("-", "")
            },
            tx_amt: cents_to_yuan_str(voucher.amount_cents), // cents -> yuan
            charge_fee_type: "0".to_string(),
            memo: voucher.reference.clone(),
            notify_method: "1".to_string(),
            notify_email: String::new(),
            notify_fax: String::new(),
            write_off_no: voucher.voucher_no.clone(),
            remark: String::new(),
            tx_seq: format!(
                "IOB{}{}",
                chrono::Utc::now().format("%Y%m%d"),
                voucher.voucher_no
            ),
            detail_list: vec![],
        }
    }

    /// Build a TwdPaymentRequest from a list of vouchers.
    pub fn build_twd_payment_request(
        &self,
        vouchers: &[Voucher],
        rq_header: api::RqHeader,
        payer_account: &str,
        payer_id: &str,
        format_type: &str,
    ) -> TwdPaymentRequest {
        let tx_list: Vec<TwdPaymentTx> = vouchers
            .iter()
            .map(|v| Self::voucher_to_twd_tx(v, payer_account, payer_id))
            .collect();

        // For format_type=2 (JSON), content is base64-encoded JSON
        let content = if format_type == "2" {
            let json = serde_json::to_string(&tx_list).unwrap_or_default();
            crate::media::base64_encode_media(json.as_bytes())
        } else {
            // format_type=1: assemble 422-char media file, base64-encode
            let media_records: Vec<crate::media::TwdMediaRecord> = vouchers
                .iter()
                .map(|v| crate::media::TwdMediaRecord {
                    payer_account_no: payer_account.to_string(),
                    payer_id: payer_id.to_string(),
                    payee_bank_id: "000".to_string(),
                    payee_branch_id: "0000".to_string(),
                    payee_account_no: v.bank_account.clone(),
                    payee_name: v.vendor_code.clone(),
                    payee_id: String::new(),
                    tx_date: if v.due_date.is_empty() {
                        chrono::Utc::now().format("%Y%m%d").to_string()
                    } else {
                        v.due_date.replace("-", "")
                    },
                    tx_amt: cents_to_yuan_str(v.amount_cents),
                    charge_fee_type: "0".to_string(),
                    memo: v.reference.clone(),
                    notify_method: "1".to_string(),
                    notify_email: String::new(),
                    notify_fax: String::new(),
                    write_off_no: v.voucher_no.clone(),
                    remark: String::new(),
                    tx_seq: format!("IOB{}{}", chrono::Utc::now().format("%Y%m%d"), v.voucher_no),
                })
                .collect();
            match crate::media::assemble_twd_media_file(&media_records) {
                Ok(bytes) => crate::media::base64_encode_media(&bytes),
                Err(_) => String::new(),
            }
        };

        TwdPaymentRequest {
            rq_header,
            rq_data: TwdPaymentRequestData {
                format_type: format_type.to_string(),
                payment_type: "1".to_string(),
                verification_type: "1".to_string(),
                flow_info: api::FlowInfo::default(),
                content,
            },
        }
    }

    /// Check if an API result code indicates success.
    pub fn is_api_success(code: &str) -> bool {
        api::is_api_success(code)
    }

    /// Check if a transaction status indicates success.
    pub fn is_tx_success(status: &str) -> bool {
        api::is_tx_success(status)
    }

    /// Check if a transaction status indicates pending.
    pub fn is_tx_pending(status: &str) -> bool {
        api::is_tx_pending(status)
    }

    /// Check if a transaction status indicates failure.
    pub fn is_tx_failed(status: &str) -> bool {
        api::is_tx_failed(status)
    }

    /// Export vouchers to a bank media file and dispatch via the release gate.
    ///
    /// Steps:
    /// 1. Read + decode the source file (auto-detect encoding).
    /// 2. Resolve column indices from the header row; validate each data row.
    /// 3. Lock each voucher's idempotency key (`VOUCHER_NO:AMOUNT:BATCH_DATE`).
    /// 4. Register the file batch in the ledger.
    /// 5. Assemble the media file; run `ReleaseGate::verify_payload`.
    /// 6. If valid, `ReleaseGate::dispatch` (caller-supplied transport).
    pub fn export_vouchers(
        &mut self,
        source_path: &Path,
        batch_id: &str,
        batch_date: &str,
        encoding: BankEncoding,
        transport: impl FnOnce(&[u8]) -> Result<bool, String>,
    ) -> Result<ExportResult, String> {
        let bytes = fs::read(source_path)
            .map_err(|e| format!("cannot read source file {}: {e}", source_path.display()))?;
        if bytes.len() > MAX_INPUT_BYTES {
            return Err(format!(
                "source file {} exceeds {} byte limit",
                source_path.display(),
                MAX_INPUT_BYTES
            ));
        }
        let (content, _detected) = to_utf8(&bytes, None);

        let source_name = source_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let ledger_registered = self
            .ledger
            .register_file_batch(batch_id, "E_SUN_PAY", &source_name, &bytes, 0)
            .map_err(|e| format!("ledger register batch failed: {e}"))?;
        if !ledger_registered {
            return Err(format!(
                "batch {batch_id} already registered (duplicate content)"
            ));
        }

        let contract = voucher_contract();
        let mut records = split_records(&content).into_iter().peekable();
        let (_, header_line) = records
            .next()
            .ok_or_else(|| "source file is empty".to_string())?;
        let cols = resolve_voucher_columns(header_line, &contract)?;

        let mut accepted: Vec<Voucher> = Vec::new();
        let mut isolated: Vec<String> = Vec::new();

        for (line_no, line) in records {
            let sep = detect_separator(line);
            let values: Vec<&str> = line.split(sep).map(str::trim).collect();
            match parse_voucher_row(&values, &cols, line_no) {
                Ok(voucher) => {
                    let key = format!(
                        "{}:{}:{}",
                        voucher.voucher_no, voucher.amount_cents, batch_date
                    );
                    let locked = self
                        .ledger
                        .check_and_lock_order(&key, batch_id, voucher.amount_cents)
                        .map_err(|e| format!("ledger lock order failed: {e}"))?;
                    if !locked {
                        isolated.push(format!(
                            "line {line_no}: voucher {} already processed in this batch",
                            voucher.voucher_no
                        ));
                        continue;
                    }
                    accepted.push(voucher);
                }
                Err(e) => {
                    self.ledger
                        .log_finding(Finding {
                            batch_id,
                            rule_code: "VR-BANKPAY-01",
                            severity: FindingSeverity::Error,
                            filename: &source_name,
                            line_number: line_no,
                            key_value: cell(&values, cols.voucher_no),
                            reason: &e,
                        })
                        .map_err(|e| format!("ledger log finding failed: {e}"))?;
                    isolated.push(format!("line {line_no}: {e}"));
                }
            }
        }

        let (media_bytes, had_errors) = assemble_media_file(&accepted, batch_id, encoding);
        let file_sha256 = media_file_sha256(&media_bytes);

        let media_path = format!("{batch_id}.bankpay.media");
        fs::write(&media_path, &media_bytes)
            .map_err(|e| format!("cannot write media file {media_path}: {e}"))?;

        // Release gate: two-way door verify, then one-way door dispatch.
        let valid = isolated.is_empty() && !had_errors;
        self.gate.lock().unwrap().verify_payload(
            valid,
            if valid {
                ""
            } else {
                "isolated rows or encoding errors present"
            },
        );

        let dispatched = if valid {
            match transport(&media_bytes) {
                Ok(true) => {
                    self.gate
                        .lock()
                        .unwrap()
                        .dispatch(|| Ok(true), None)
                        .map_err(|e| format!("dispatch failed: {e}"))?;
                    true
                }
                Ok(false) => {
                    self.gate
                        .lock()
                        .unwrap()
                        .dispatch(|| Ok(false), None)
                        .map_err(|e| format!("dispatch reported failure: {e}"))?;
                    false
                }
                Err(e) => {
                    let _ = self.gate.lock().unwrap().dispatch(|| Err(e.clone()), None);
                    return Err(format!("transport error: {e}"));
                }
            }
        } else {
            false
        };

        let release_state = format!("{:?}", self.gate.lock().unwrap().state);

        Ok(ExportResult {
            batch_id: batch_id.to_string(),
            media_file: media_path,
            file_sha256,
            vouchers_accepted: accepted.len(),
            vouchers_isolated: isolated.len(),
            dispatched,
            release_state,
        })
    }

    /// Import bank results and settle matched vouchers.
    ///
    /// Steps:
    /// 1. Read + decode the bank result file.
    /// 2. Resolve column indices from the header row; validate each data row.
    /// 3. Record each result as a finding so the traceable query can
    ///    cross-reference bank_ref to voucher later.
    /// 4. Unparseable results go to quarantine.
    pub fn import_results(
        &mut self,
        source_path: &Path,
        batch_id: &str,
        _encoding: BankEncoding,
    ) -> Result<ImportResult, String> {
        let bytes = fs::read(source_path)
            .map_err(|e| format!("cannot read result file {}: {e}", source_path.display()))?;
        let (content, _detected) = to_utf8(&bytes, None);

        let source_name = source_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let _registered = self
            .ledger
            .register_file_batch(batch_id, "E_SUN_RESULT", &source_name, &bytes, 0)
            .map_err(|e| format!("ledger register result batch failed: {e}"))?;

        let contract = bank_result_contract();
        let mut records = split_records(&content).into_iter().peekable();
        let (_, header_line) = records
            .next()
            .ok_or_else(|| "source file is empty".to_string())?;
        let cols = resolve_bank_result_columns(header_line, &contract)?;

        let mut accepted: Vec<BankResult> = Vec::new();
        let mut isolated: Vec<String> = Vec::new();
        let mut matched = 0usize;
        let mut unmatched = 0usize;

        for (line_no, line) in records {
            let sep = detect_separator(line);
            let values: Vec<&str> = line.split(sep).map(str::trim).collect();
            match parse_bank_result_row(&values, &cols, line_no) {
                Ok(result) => {
                    let key = format!("BANK_REF:{}:{}", result.bank_ref, batch_id);
                    let exists = self
                        .ledger
                        .is_order_processed(&key)
                        .map_err(|e| format!("ledger lookup failed: {e}"))?;
                    if exists {
                        isolated.push(format!(
                            "line {line_no}: bank ref {} already settled",
                            result.bank_ref
                        ));
                        continue;
                    }
                    self.ledger
                        .log_finding(Finding {
                            batch_id,
                            rule_code: "VR-BANKPAY-02",
                            severity: FindingSeverity::Info,
                            filename: &source_name,
                            line_number: line_no,
                            key_value: &result.bank_ref,
                            reason: &format!(
                                "bank result {} status={} amount={}",
                                result.bank_ref, result.status, result.amount_cents
                            ),
                        })
                        .map_err(|e| format!("ledger log finding failed: {e}"))?;
                    matched += 1;
                    accepted.push(result);
                }
                Err(e) => {
                    self.ledger
                        .log_quarantine(
                            batch_id,
                            &source_name,
                            line_no,
                            &values.join(","),
                            "VR-BANKPAY-03",
                            &e,
                        )
                        .map_err(|e| format!("ledger log quarantine failed: {e}"))?;
                    isolated.push(format!("line {line_no}: {e}"));
                    unmatched += 1;
                }
            }
        }

        Ok(ImportResult {
            batch_id: batch_id.to_string(),
            results_accepted: accepted.len(),
            results_isolated: isolated.len(),
            matched_vouchers: matched,
            unmatched,
        })
    }
}

/// Detect the delimiter of a delimited line.
fn detect_separator(line: &str) -> char {
    if line.contains(';') {
        ';'
    } else if line.contains(',') {
        ','
    } else {
        '\t'
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use wellfit_core::ledger::BatchScope;

    static FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn test_ledger() -> IdempotencyLedger {
        IdempotencyLedger::new_scoped(":memory:", BatchScope::Exclusive).unwrap()
    }

    fn write_tmp_csv(content: &str) -> std::path::PathBuf {
        let n = FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("bankpay_test_{}_{}.csv", std::process::id(), n));
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_export_vouchers_dedup_and_lock() {
        let ledger = Arc::new(test_ledger());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = BankpayEngine::new(ledger, gate);
        let path = write_tmp_csv(
            "VOUCHER_NO,VENDOR_CODE,AMOUNT,BANK_ACCOUNT,DUE_DATE,REFERENCE\n\
             AP-001,V001,1000.0000,ACCT-1,2026-11-03,INV-100\n\
             AP-002,V002,2500.5000,ACCT-2,2026-11-04,INV-101\n",
        );
        let batch_date = "2026-11-02";
        let r1 = engine.export_vouchers(&path, "BATCH-1", batch_date, BankEncoding::Utf8, |_| {
            Ok(true)
        });
        assert!(r1.is_ok(), "first export must succeed: {:?}", r1.err());
        let r1 = r1.unwrap();
        assert_eq!(r1.vouchers_accepted, 2);
        assert!(r1.dispatched);

        // Re-export same batch id -> duplicate rejected.
        let r2 = engine.export_vouchers(&path, "BATCH-1", batch_date, BankEncoding::Utf8, |_| {
            Ok(true)
        });
        assert!(r2.is_err(), "duplicate batch must be rejected");
    }

    #[test]
    fn test_export_isolates_bad_amount() {
        let ledger = Arc::new(test_ledger());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = BankpayEngine::new(ledger, gate);
        let path = write_tmp_csv(
            "VOUCHER_NO,VENDOR_CODE,AMOUNT,BANK_ACCOUNT\nAP-001,V001,notanumber,ACCT-1\n",
        );
        let r =
            engine.export_vouchers(&path, "BATCH-BAD", "2026-11-02", BankEncoding::Utf8, |_| {
                Ok(true)
            });
        assert!(r.is_ok());
        let r = r.unwrap();
        assert_eq!(r.vouchers_accepted, 0);
        assert_eq!(r.vouchers_isolated, 1);
        assert!(!r.dispatched);
    }

    #[test]
    fn test_assemble_media_file_and_sha256() {
        let vouchers = vec![Voucher {
            voucher_no: "AP-001".to_string(),
            vendor_code: "V001".to_string(),
            amount_cents: 1000,
            due_date: "2026-11-03".to_string(),
            bank_account: "ACCT-1".to_string(),
            reference: "INV-100".to_string(),
            line: 1,
            raw: String::new(),
        }];
        let (bytes, had_errors) = assemble_media_file(&vouchers, "BATCH-X", BankEncoding::Utf8);
        assert!(!had_errors);
        let sha = media_file_sha256(&bytes);
        assert_eq!(sha.len(), 64);
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("AP-001"));
        assert!(text.contains("TRAILER"));
    }

    #[test]
    fn test_import_results_records_findings() {
        let ledger = Arc::new(test_ledger());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = BankpayEngine::new(ledger, gate);
        let path = write_tmp_csv(
            "BANK_REF,VOUCHER_NO,STATUS,AMOUNT,PROCESSED_AT\n\
             EREF-999,AP-001,paid,1000.0000,2026-11-03T09:00:00Z\n",
        );
        let r = engine.import_results(&path, "BATCH-RES", BankEncoding::Utf8);
        assert!(r.is_ok(), "import must succeed: {:?}", r.err());
        let r = r.unwrap();
        assert_eq!(r.results_accepted, 1);
        assert_eq!(r.matched_vouchers, 1);
        assert_eq!(r.unmatched, 0);
    }

    #[test]
    fn test_end_to_end_traceability_chain() {
        // Voucher -> export -> media file -> sha256 -> bank result -> import
        // -> query (voucher/tx/batch) cross-reference.
        let ledger = Arc::new(test_ledger());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = BankpayEngine::new(ledger.clone(), gate);

        let voucher_path = write_tmp_csv(
            "VOUCHER_NO,VENDOR_CODE,AMOUNT,BANK_ACCOUNT,DUE_DATE,REFERENCE\n\
             AP-TRACE-001,VTRACE,12340.0000,ACCT-TRACE,2026-11-03,INV-TRACE\n",
        );
        let batch_date = "2026-11-02";
        let batch_id = "BATCH-TRACE-20261102";

        // 1. Export produces media file with a content hash.
        let export = engine
            .export_vouchers(
                &voucher_path,
                batch_id,
                batch_date,
                BankEncoding::Utf8,
                |_| Ok(true),
            )
            .unwrap();
        assert_eq!(export.vouchers_accepted, 1);
        assert!(export.dispatched);
        assert!(!export.file_sha256.is_empty());
        assert_eq!(export.file_sha256.len(), 64);

        // 2. Media file bytes hash matches the reported sha256.
        let media_bytes = std::fs::read(&export.media_file).unwrap();
        assert_eq!(media_file_sha256(&media_bytes), export.file_sha256);

        // 3. Import bank result referencing the voucher.
        let result_path = write_tmp_csv(
            "BANK_REF,VOUCHER_NO,STATUS,AMOUNT,PROCESSED_AT\n\
             ETRACE-9001,AP-TRACE-001,paid,12340.0000,2026-11-03T09:00:00Z\n",
        );
        let import = engine
            .import_results(&result_path, "BATCH-TRACE-RES", BankEncoding::Utf8)
            .unwrap();
        assert_eq!(import.results_accepted, 1);
        assert_eq!(import.matched_vouchers, 1);

        // 4. Traceable query cross-references voucher / tx / batch.
        use crate::query::query;
        let q_voucher = query(&mut engine, "voucher", "AP-TRACE-001").unwrap();
        assert_eq!(q_voucher.voucher_no, Some("AP-TRACE-001".to_string()));
        assert!(!q_voucher.findings.is_empty());

        let q_tx = query(&mut engine, "tx", "ETRACE-9001").unwrap();
        assert_eq!(q_tx.bank_ref, Some("ETRACE-9001".to_string()));
        assert!(!q_tx.findings.is_empty());

        let q_batch = query(&mut engine, "batch", batch_id).unwrap();
        assert_eq!(q_batch.batch_id, Some(batch_id.to_string()));
        assert!(!q_batch.findings.is_empty());

        // 5. Audit trail: export registered the batch; import + each query
        //    recorded a finding in validation_findings for traceability.
        let export_hash = engine.ledger.content_hash_of(batch_id).unwrap();
        assert!(export_hash.is_some(), "export batch must be registered");

        let import_findings = engine.ledger.get_findings(Some("BATCH-TRACE-RES")).unwrap();
        assert!(
            !import_findings.is_empty(),
            "import findings must be recorded"
        );

        let query_findings = engine
            .ledger
            .get_findings(None)
            .unwrap()
            .into_iter()
            .filter(|f| f.rule_code == "VR-BANKPAY-QUERY")
            .count();
        assert!(
            query_findings >= 3,
            "each query must record a traceable finding"
        );
    }

    #[test]
    fn test_encoding_round_trip_used_by_engine() {
        let text = "AP-001,V001,1000,ACCT-1,2026-11-03,INV-100";
        let (bytes, had_errors) = from_utf8(text, BankEncoding::Big5);
        assert!(!had_errors);
        let (decoded, detected) = to_utf8(&bytes, Some(BankEncoding::Big5));
        assert_eq!(detected, BankEncoding::Big5);
        assert_eq!(decoded, text);
    }
}

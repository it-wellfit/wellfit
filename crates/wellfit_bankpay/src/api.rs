#![forbid(unsafe_code)]

//! E.Sun Bank ERP integration API models (V2.8: 8 endpoints).
//!
//! Serde structs for the request/response envelopes used by the
//! `銀企直連安控元件` (security component) on port 8443.
//!
//! Reference: `【玉山銀行】企業ERP銀企直連串接_技術文件V2.8_v1.pdf`
//!
//! Field mapping: each endpoint's request/response fields are traced to
//! the V2.8 technical document (`04-interface-specification.md` sections
//! 2-9) and the business workflow (`02-business-workflow.md` 3-5).
//! All amounts use `String` representation (no float); parse as
//! fixed-precision integers at the boundary.
//!
//! ADR lock (Phase A): `foreignTxList` and status-result lists are
//! serialized with camelCase field names exactly as documented in 04 §5-§9.
//! Amount fields are strings per the V2.8 contract (decimal type maps to
//! string representation at the transport boundary).

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Common envelope
// ---------------------------------------------------------------------------

/// Request header envelope (rqHeader).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RqHeader {
    /// Message ID: 統編(8-10) + YYYYMMDD + 7-digit sequence.
    #[serde(rename = "msgId")]
    pub msg_id: String,
    /// Request time: `YYYY/MM/DD HH:MM:SS` (UTC+8).
    #[serde(rename = "requestTime")]
    pub request_time: String,
    /// Enterprise group tax ID (8-10 digits).
    #[serde(rename = "customerIxd")]
    pub customer_ixd: String,
    /// Agreed SourceSystemId.
    #[serde(rename = "sourceSystem")]
    pub source_system: String,
}

/// Response header envelope (rsHeader).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsHeader {
    /// Echoed msgId.
    #[serde(rename = "msgId")]
    pub msg_id: String,
    /// Component tracking sequence (40 chars).
    #[serde(rename = "logId")]
    pub log_id: String,
    /// Reply time: `YYYY/MM/DD HH:MM:SS`.
    #[serde(rename = "responseTime")]
    pub response_time: String,
}

/// Response status (rsStatus).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsStatus {
    /// Result code. `910000` = success; all others = failure.
    pub code: String,
    /// Description (max 60 chars).
    pub message: String,
    /// Per-transaction error detail (semicolon-separated). Present when code=910071.
    #[serde(rename = "moreInfo", default)]
    pub more_info: String,
}

/// Common response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    #[serde(rename = "rsHeader")]
    pub rs_header: RsHeader,
    #[serde(rename = "rsStatus")]
    pub rs_status: RsStatus,
    #[serde(rename = "rsData", default)]
    pub rs_data: Option<T>,
}

impl<T> ApiResponse<T> {
    /// Whether the API call succeeded (code == "910000").
    pub fn is_success(&self) -> bool {
        self.rs_status.code == "910000"
    }
}

// ---------------------------------------------------------------------------
// TWD Payment Release API
// POST /payment/createTwdPaymentInstruction
// ---------------------------------------------------------------------------

/// A single TWD payment transaction (twdTxList item).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwdPaymentTx {
    /// Payer E.Sun account (≥13 digits, max 16).
    #[serde(rename = "payerAccountNo")]
    pub payer_account_no: String,
    /// Payer tax ID (≥10 digits, max 15).
    #[serde(rename = "payerId")]
    pub payer_id: String,
    /// Payee bank code (3 digits).
    #[serde(rename = "payeeBankId")]
    pub payee_bank_id: String,
    /// Payee bank branch (4 digits).
    #[serde(rename = "payeeBranchId")]
    pub payee_branch_id: String,
    /// Payee account (max 16).
    #[serde(rename = "payeeAccountNo")]
    pub payee_account_no: String,
    /// Payee name (max 80, full-width 2 each, max 39 full-width chars).
    #[serde(rename = "payeeName")]
    pub payee_name: String,
    /// Payee tax ID (max 15).
    #[serde(rename = "payeeId", default)]
    pub payee_id: String,
    /// Payment date (YYYYMMDD).
    #[serde(rename = "txDate")]
    pub tx_date: String,
    /// Payment amount (>0, max 13 digits).
    #[serde(rename = "txAmt")]
    pub tx_amt: String,
    /// Fee burden: 0=external (payer), 1=internal (payee). Default 0.
    #[serde(rename = "chargeFeeType")]
    pub charge_fee_type: String,
    /// Memo (max 80, full-width 2 each, max 39 full-width chars).
    #[serde(rename = "memo", default)]
    pub memo: String,
    /// Notify method: 1=none, 2=email, 3=fax, 9=all.
    #[serde(rename = "notifyMethod", default)]
    pub notify_method: String,
    /// Notify email (semicolon-separated, max 400).
    #[serde(rename = "notifyEmail", default)]
    pub notify_email: String,
    /// Notify fax (min 10 including area code, max 20).
    #[serde(rename = "notifyFax", default)]
    pub notify_fax: String,
    /// Write-off number (max 40).
    #[serde(rename = "writeOffNo", default)]
    pub write_off_no: String,
    /// Remark (max 38, full-width 2 each, max 18 full-width chars).
    #[serde(rename = "remark", default)]
    pub remark: String,
    /// ERP transaction sequence (max 22, must be unique).
    #[serde(rename = "txSeq")]
    pub tx_seq: String,
    /// Invoice detail list.
    #[serde(rename = "detailList", default)]
    pub detail_list: Vec<TwdPaymentDetail>,
}

/// Invoice detail (detailList item).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TwdPaymentDetail {
    /// Invoice date (YYYYMMDD, fixed 8).
    #[serde(rename = "invoiceDate", default)]
    pub invoice_date: String,
    /// Invoice number (max 20, alphanumeric).
    #[serde(rename = "invoiceNo", default)]
    pub invoice_no: String,
    /// Payable amount (18.2).
    #[serde(rename = "invoiceAmt", default)]
    pub invoice_amt: String,
    /// Discount/return amount (18.2).
    #[serde(rename = "invoiceReturn", default)]
    pub invoice_return: String,
    /// Actual payment amount (18.2).
    #[serde(rename = "invoiceActualAmt", default)]
    pub invoice_actual_amt: String,
    /// Invoice remark (max 40).
    #[serde(rename = "invoiceRemark", default)]
    pub invoice_remark: String,
}

/// Flow info for bank-verification type (verificationType=2).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlowInfo {
    /// Net-banking operator name (max 32).
    #[serde(rename = "operatorName", default)]
    pub operator_name: String,
    /// Net-banking flow name (max 64, required if type 2).
    #[serde(rename = "flowName", default)]
    pub flow_name: String,
    /// Flow key (max 18).
    #[serde(rename = "flowKey", default)]
    pub flow_key: Option<i64>,
}

/// TWD payment request data (rqData).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwdPaymentRequestData {
    /// 1=file (base64 media), 2=json.
    #[serde(rename = "formatType")]
    pub format_type: String,
    /// 1=one debit one credit, 2=one debit many credits.
    #[serde(rename = "paymentType")]
    pub payment_type: String,
    /// 1=ERP verification, 2=bank verification.
    #[serde(rename = "verificationType")]
    pub verification_type: String,
    /// Flow info (required if verificationType=2).
    #[serde(rename = "flowInfo", default)]
    pub flow_info: FlowInfo,
    /// Base64 content (file or JSON).
    pub content: String,
}

/// Full TWD payment request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwdPaymentRequest {
    #[serde(rename = "rqHeader")]
    pub rq_header: RqHeader,
    #[serde(rename = "rqData")]
    pub rq_data: TwdPaymentRequestData,
}

/// TWD payment response data (rsData).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TwdPaymentResponseData {
    /// Transaction count.
    #[serde(rename = "dataCount", default)]
    pub data_count: String,
    /// Transaction records.
    #[serde(rename = "txRecordList", default)]
    pub tx_record_list: Vec<TwdPaymentRecord>,
}

/// TWD payment transaction record (response).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TwdPaymentRecord {
    /// ERP transaction sequence.
    #[serde(rename = "txSeq", default)]
    pub tx_seq: String,
    /// Whether txDate was deferred: 0=no, 1=yes.
    #[serde(rename = "autoDelay", default)]
    pub auto_delay: String,
    /// Payer account (13 digits).
    #[serde(rename = "payerAccountNo", default)]
    pub payer_account_no: String,
    /// Payer tax ID (15 digits).
    #[serde(rename = "payerId", default)]
    pub payer_id: String,
    /// Payee bank code (3 digits).
    #[serde(rename = "payeeBankId", default)]
    pub payee_bank_id: String,
    /// Payee branch (4 digits).
    #[serde(rename = "payeeBranchId", default)]
    pub payee_branch_id: String,
    /// Payee account (16 digits).
    #[serde(rename = "payeeAccountNo", default)]
    pub payee_account_no: String,
    /// Payee name (80 chars).
    #[serde(rename = "payeeName", default)]
    pub payee_name: String,
    /// Payee tax ID (15 digits).
    #[serde(rename = "payeeId", default)]
    pub payee_id: String,
    /// Payment date (YYYYMMDD).
    #[serde(rename = "txDate", default)]
    pub tx_date: String,
    /// Payment amount (13 digits).
    #[serde(rename = "txAmt", default)]
    pub tx_amt: String,
    /// Fee burden type.
    #[serde(rename = "chargeFeeType", default)]
    pub charge_fee_type: String,
    /// Memo (80 chars).
    #[serde(rename = "memo", default)]
    pub memo: String,
    /// Notify method.
    #[serde(rename = "notifyMethod", default)]
    pub notify_method: String,
    /// Notify email.
    #[serde(rename = "notifyEmail", default)]
    pub notify_email: String,
    /// Notify fax.
    #[serde(rename = "notifyFax", default)]
    pub notify_fax: String,
    /// Write-off number.
    #[serde(rename = "writeOffNo", default)]
    pub write_off_no: String,
    /// Remark.
    #[serde(rename = "remark", default)]
    pub remark: String,
    /// Invoice details.
    #[serde(rename = "detailList", default)]
    pub detail_list: Vec<TwdPaymentDetail>,
}

// ---------------------------------------------------------------------------
// TWD Transaction Status Query API
// POST /payment/retrieveTwdPaymentInstructionResult
// ---------------------------------------------------------------------------

/// TWD transaction status query request data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwdStatusQueryRequestData {
    /// ERP payment batch number (payment unified number).
    #[serde(rename = "paymentNo")]
    pub payment_no: String,
    /// ERP transaction sequence.
    #[serde(rename = "txSeq")]
    pub tx_seq: String,
}

/// Full TWD status query request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwdStatusQueryRequest {
    #[serde(rename = "rqHeader")]
    pub rq_header: RqHeader,
    #[serde(rename = "rqData")]
    pub rq_data: TwdStatusQueryRequestData,
}

/// TWD transaction status response data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TwdStatusResponseData {
    /// Transaction status code (0000=success, 1000-1008=pending, 4000=failed, 9999=no data).
    #[serde(rename = "txStatus", default)]
    pub tx_status: String,
    /// Bank transaction reference.
    #[serde(rename = "bankRef", default)]
    pub bank_ref: String,
    /// Transaction amount.
    #[serde(rename = "txAmt", default)]
    pub tx_amt: String,
    /// Transaction date.
    #[serde(rename = "txDate", default)]
    pub tx_date: String,
    /// Error message.
    #[serde(rename = "errorMessage", default)]
    pub error_message: String,
}

// ---------------------------------------------------------------------------
// Transaction Status Constants
// ---------------------------------------------------------------------------

/// Transaction status codes from E.Sun.
pub mod tx_status {
    /// Transaction succeeded.
    pub const SUCCESS: &str = "0000";
    /// Pending / processing (1000-1008).
    pub const PENDING_RANGE: (u32, u32) = (1000, 1008);
    /// Foreign currency related.
    pub const FX_RELATED: [&str; 3] = ["1030", "1031", "1032"];
    /// FX pre-check.
    pub const FX_PRECHECK: [&str; 2] = ["3000", "3001"];
    /// Transaction failed.
    pub const FAILED: &str = "4000";
    /// No data.
    pub const NO_DATA: &str = "9999";
}

/// Whether a status code indicates success.
pub fn is_tx_success(code: &str) -> bool {
    code == tx_status::SUCCESS
}

/// Whether a status code indicates pending/processing.
pub fn is_tx_pending(code: &str) -> bool {
    if let Ok(n) = code.parse::<u32>() {
        n >= tx_status::PENDING_RANGE.0 && n <= tx_status::PENDING_RANGE.1
    } else {
        false
    }
}

/// Whether a status code indicates failure.
pub fn is_tx_failed(code: &str) -> bool {
    code == tx_status::FAILED
}

// ---------------------------------------------------------------------------
// API Result Code Constants
// ---------------------------------------------------------------------------

/// API result codes from E.Sun.
pub mod api_result {
    /// Success — rsData is present.
    pub const SUCCESS: &str = "910000";
    /// Field validation error — moreInfo contains per-tx error detail.
    pub const FIELD_ERROR: &str = "910071";
    /// Unknown error.
    pub const UNKNOWN: &str = "999999";
}

/// Whether an API result code indicates success.
pub fn is_api_success(code: &str) -> bool {
    code == api_result::SUCCESS
}

// ---------------------------------------------------------------------------
// API #3 — Transaction Detail Query (Statement)
// POST /account/retrieveStatements
// ---------------------------------------------------------------------------
//
// Field mapping (V2.8: 04-interface-specification.md §4).
//
// ADR lock (Phase A, §4 of the master plan):
// - `sTxDate`/`eTxDate` are YYYYMMDD (length 8); `sTxTime`/`eTxTime` are
//   HHMMSSFF (length 8; defaults `00000000`/`23595999`).
// - `queryStartIndex` is a string (length 5), first = 1. Pagination: empty
//   `firstDeptTxDtlSeqNo` on the first call, then pass the last row's
//   `deptTxDtlSeqNo`.
//
// ADR lock for foreign-currency structured lists (Phase A):
// `foreignTxList` and FX status entries are serialized from this module.
// `FxResultEntry` is a documented subset (04 §7.2) of the FX result shape;
// full appendix detail can extend it without breaking existing fields.

/// Statement query request data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementQueryRequestData {
    /// Company tax ID owning the account (8-10 digits).
    #[serde(rename = "companyId")]
    pub company_id: String,
    /// E.Sun account number (13 digits).
    #[serde(rename = "demandDeptAcc")]
    pub demand_dept_acc: String,
    /// Start date (YYYYMMDD).
    #[serde(rename = "sTxDate")]
    pub s_tx_date: String,
    /// End date (YYYYMMDD).
    #[serde(rename = "eTxDate")]
    pub e_tx_date: String,
    /// Start time (HHMMSSFF, default `00000000`).
    #[serde(rename = "sTxTime", default)]
    pub s_tx_time: String,
    /// End time (HHMMSSFF, default `23595999`).
    #[serde(rename = "eTxTime", default)]
    pub e_tx_time: String,
    /// Currency (blank = all).
    #[serde(rename = "curr", default)]
    pub curr: String,
    /// Check/voucher number (optional).
    #[serde(rename = "dbCkn", default)]
    pub db_ckn: String,
    /// Enterprise asset code (optional).
    #[serde(rename = "idNoAssetCode", default)]
    pub id_no_asset_code: String,
    /// Ordering: `1`=newest first, `2`=oldest first.
    #[serde(rename = "orderDescType")]
    pub order_desc_type: String,
    /// Start index for paging (1-based, length 5).
    #[serde(rename = "queryStartIndex")]
    pub query_start_index: String,
    /// Positioning sequence number (first call blank).
    #[serde(rename = "firstDeptTxDtlSeqNo", default)]
    pub first_dept_tx_dtl_seq_no: String,
}

/// Full statement query request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatementQueryRequest {
    #[serde(rename = "rqHeader")]
    pub rq_header: RqHeader,
    #[serde(rename = "rqData")]
    pub rq_data: StatementQueryRequestData,
}

/// Statement transaction detail row.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeptTxDtl {
    /// Transaction detail sequence number (19 chars).
    #[serde(rename = "deptTxDtlSeqNo", default)]
    pub dept_tx_dtl_seq_no: String,
    /// E.Sun account (13 chars).
    #[serde(rename = "demandDeptAcc", default)]
    pub demand_dept_acc: String,
    /// Currency.
    #[serde(rename = "curr", default)]
    pub curr: String,
    /// Actual transaction date (YYYYMMDD).
    #[serde(rename = "txDate", default)]
    pub tx_date: String,
    /// Bank accounting (booking) date.
    #[serde(rename = "bookingDate", default)]
    pub booking_date: String,
    /// Exchange rate (12.6).
    #[serde(rename = "exchgRate", default)]
    pub exchg_rate: String,
    /// Actual transaction time.
    #[serde(rename = "txTime", default)]
    pub tx_time: String,
    /// Transaction amount (18.2 decimal; string at transport boundary).
    #[serde(rename = "txAmt", default)]
    pub tx_amt: String,
    /// Check number (10 chars).
    #[serde(rename = "chkNo", default)]
    pub chk_no: String,
    /// `DB`=debit, `CR`=credit.
    #[serde(rename = "debitCredit", default)]
    pub debit_credit: String,
    /// Balance before transaction (18.2).
    #[serde(rename = "blncBeforeTx", default)]
    pub blnc_before_tx: String,
    /// Balance after transaction (18.2).
    #[serde(rename = "blncAfterTx", default)]
    pub blnc_after_tx: String,
    /// Enterprise ID code (5 chars).
    #[serde(rename = "idNoOfCom", default)]
    pub id_no_of_com: String,
    /// Transaction summary (14 chars).
    #[serde(rename = "summary", default)]
    pub summary: String,
    /// Transaction detail remark (40 chars).
    #[serde(rename = "txDtlsRemark", default)]
    pub tx_dtls_remark: String,
    /// Type: `1`=cash, `2`=transfer, `3`=post-dated, `4`=pre-dated check.
    #[serde(rename = "txType", default)]
    pub tx_type: String,
    /// `N`=normal, `Y`=reversal.
    #[serde(rename = "rvrsTxMark", default)]
    pub rvrs_tx_mark: String,
    /// Counterparty bank code (3 digits).
    #[serde(rename = "oppositeBankCode", default)]
    pub opposite_bank_code: String,
    /// Counterparty bank account (16 chars).
    #[serde(rename = "oppositeAccount", default)]
    pub opposite_account: String,
}

/// Statement query response data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatementResponseData {
    /// Transaction detail rows (max 500 per page).
    #[serde(rename = "deptTxDtlList", default)]
    pub dept_tx_dtl_list: Vec<DeptTxDtl>,
}

// ---------------------------------------------------------------------------
// API #4 — FX Pre-Check (Validation)
// POST /payment/validateFxPaymentInstruction
// ---------------------------------------------------------------------------

/// A foreign-currency transaction (foreignTxList item).
///
/// Field mapping (V2.8: 04 §5.2). Amount fields are strings (decimal type
/// maps to string representation at the transport boundary, no float).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ForeignTx {
    /// ERP txSeq (max 22, unique).
    #[serde(rename = "txSeq")]
    pub tx_seq: String,
    /// Transaction date (YYYYMMDD, today or later).
    #[serde(rename = "txDate")]
    pub tx_date: String,
    /// Payer tax ID (8-15 digits).
    #[serde(rename = "payerId")]
    pub payer_id: String,
    /// Payer account (12-17 digits).
    #[serde(rename = "payerAccountNo")]
    pub payer_account_no: String,
    /// Payer currency (3 chars).
    #[serde(rename = "payerCurrency")]
    pub payer_currency: String,
    /// Payer amount (decimal 15.2; alternative to payeeAmount).
    #[serde(rename = "payerAmount", default)]
    pub payer_amount: String,
    /// Payee account (34 chars max; IBAN for European beneficiaries).
    #[serde(rename = "payeeAccountNo")]
    pub payee_account_no: String,
    /// Payee currency (3 chars).
    #[serde(rename = "payeeCurrency")]
    pub payee_currency: String,
    /// Payee amount (decimal 15.2; alternative to payerAmount).
    #[serde(rename = "payeeAmount", default)]
    pub payee_amount: String,
    /// Payee name (140 chars max; required for cross-border FX).
    #[serde(rename = "payeeName", default)]
    pub payee_name: String,
    /// Payee bank SWIFT code (8/11 chars) or bank name.
    #[serde(rename = "payeeBankSwiftCode", default)]
    pub payee_bank_swift_code: String,
    /// Payee bank country (2 chars).
    #[serde(rename = "payeeBankAddressCountry")]
    pub payee_bank_address_country: String,
    /// Remittance type: `1`=general, `2`=full to bank, `3`=beneficiary pays.
    #[serde(rename = "remittanceType")]
    pub remittance_type: String,
    /// Fee type: `0`=external, `1`=internal.
    #[serde(rename = "feeType")]
    pub fee_type: String,
    /// Fee debit account (12-17 digits; required cross-border).
    #[serde(rename = "feeDebitAccountNo", default)]
    pub fee_debit_account_no: String,
    /// Fee debit currency (3 chars).
    #[serde(rename = "feeDebitCurrency", default)]
    pub fee_debit_currency: String,
    /// Notification type: `0/1`=none, `2`=email.
    #[serde(rename = "notificationType", default)]
    pub notification_type: String,
    /// Notification info (emails, max 250).
    #[serde(rename = "notificationInfo", default)]
    pub notification_info: String,
}

/// FX pre-check / release request data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FxPaymentRequestData {
    /// `2`=json (the only supported value for FX per 04 §5-§6).
    #[serde(rename = "formatType")]
    pub format_type: String,
    /// `1`=one debit one credit, `2`=one debit many credits.
    #[serde(rename = "paymentType")]
    pub payment_type: String,
    /// `1`=ERP verification, `2`=bank verification (release only).
    #[serde(rename = "verificationType", default)]
    pub verification_type: String,
    /// Flow info (required if verificationType=2; release only).
    #[serde(rename = "flowInfo", default)]
    pub flow_info: FlowInfo,
    /// Base64(JSON) of the foreignTxList array.
    pub content: String,
}

/// Full FX pre-check / release request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FxPaymentRequest {
    #[serde(rename = "rqHeader")]
    pub rq_header: RqHeader,
    #[serde(rename = "rqData")]
    pub rq_data: FxPaymentRequestData,
}

/// FX result entry (status-result list item).
///
/// Field mapping (V2.8: 04 §7.2). This is a documented subset; additional
/// appendix fields can be added without breaking existing ones.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FxResultEntry {
    /// Transaction status code (4 chars).
    #[serde(rename = "status", default)]
    pub status: String,
    /// Status description (60 chars).
    #[serde(rename = "statusDesc", default)]
    pub status_desc: String,
    /// Bank transaction reference (16 chars).
    #[serde(rename = "transactionReferenceNo", default)]
    pub transaction_reference_no: String,
    /// FX listing rate (13 chars decimal).
    #[serde(rename = "listingRate", default)]
    pub listing_rate: String,
    /// FX dealing rate (13 chars decimal).
    #[serde(rename = "dealingRate", default)]
    pub dealing_rate: String,
    /// Fee debit amount (13 chars decimal).
    #[serde(rename = "feeDebitAmount", default)]
    pub fee_debit_amount: String,
    /// Value date (YYYYMMDD, on send).
    #[serde(rename = "valueDate", default)]
    pub value_date: String,
    /// Actual transaction date (YYYYMMDD).
    #[serde(rename = "actualTxDate", default)]
    pub actual_tx_date: String,
    /// Error detail (120 chars).
    #[serde(rename = "sysErrorDesc", default)]
    pub sys_error_desc: String,
}

/// FX/result response data (used by APIs #4, #5, #6).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FxPaymentResponseData {
    /// Aggregate pre-check status (`3000`/`3001` for API #4).
    #[serde(rename = "status", default)]
    pub status: String,
    /// Computed listing rate.
    #[serde(rename = "listingRate", default)]
    pub listing_rate: String,
    /// Computed dealing rate.
    #[serde(rename = "dealingRate", default)]
    pub dealing_rate: String,
    /// Computed fee debit amount.
    #[serde(rename = "feeDebitAmount", default)]
    pub fee_debit_amount: String,
    /// Bank transaction reference number.
    #[serde(rename = "transactionReferenceNo", default)]
    pub transaction_reference_no: String,
    /// Error detail (120 chars).
    #[serde(rename = "sysErrorDesc", default)]
    pub sys_error_desc: String,
    /// Per-transaction result entries (API #6).
    #[serde(rename = "txInfoList", default)]
    pub tx_info_list: Vec<FxResultEntry>,
}

// ---------------------------------------------------------------------------
// API #6 — FX Transaction Status Query
// POST /payment/retrieveFxPaymentInstructionResult
// ---------------------------------------------------------------------------

/// FX status query request data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FxStatusQueryRequestData {
    /// Payer tax ID (8-10 digits).
    #[serde(rename = "payerId")]
    pub payer_id: String,
    /// ERP transaction sequence (max 22).
    #[serde(rename = "txSeq")]
    pub tx_seq: String,
}

/// Full FX status query request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FxStatusQueryRequest {
    #[serde(rename = "rqHeader")]
    pub rq_header: RqHeader,
    #[serde(rename = "rqData")]
    pub rq_data: FxStatusQueryRequestData,
}

// ---------------------------------------------------------------------------
// API #7 — Account Balance Query
// POST /account/retrieveAccountBalance
// ---------------------------------------------------------------------------

/// Balance query request data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceQueryRequestData {
    /// Company tax ID owning the account (8-10 digits).
    #[serde(rename = "companyId")]
    pub company_id: String,
    /// Currency type: `1`=TWD, `2`=FX.
    #[serde(rename = "currType")]
    pub curr_type: String,
    /// E.Sun account (13 digits; blank = all accounts).
    #[serde(rename = "demandDeptAcc", default)]
    pub demand_dept_acc: String,
}

/// Full balance query request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceQueryRequest {
    #[serde(rename = "rqHeader")]
    pub rq_header: RqHeader,
    #[serde(rename = "rqData")]
    pub rq_data: BalanceQueryRequestData,
}

/// Single-account balance detail.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeptBlncDtl {
    /// E.Sun account (13 chars).
    #[serde(rename = "demandDeptAcc", default)]
    pub demand_dept_acc: String,
    /// Currency.
    #[serde(rename = "curr", default)]
    pub curr: String,
    /// Account type: `1`=checking, `2`=demand, `3`=consolidated.
    #[serde(rename = "accType", default)]
    pub acc_type: String,
    /// Net balance (18.2 decimal).
    #[serde(rename = "actBalance", default)]
    pub act_balance: String,
    /// Available balance (18.2 decimal).
    #[serde(rename = "avlBalance", default)]
    pub avl_balance: String,
    /// Today's forward-check amount (18.2).
    #[serde(rename = "todayExgAmt", default)]
    pub today_exg_amt: String,
    /// Next-business-day forward-check amount (18.2).
    #[serde(rename = "nextExgAmt", default)]
    pub next_exg_amt: String,
}

/// Balance query response data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BalanceResponseData {
    /// Balance detail rows.
    #[serde(rename = "deptBlncDtlList", default)]
    pub dept_blnc_dtl_list: Vec<DeptBlncDtl>,
}

// ---------------------------------------------------------------------------
// API #8 — TWD Payment Result Print
// POST /payment/retrieveTwdPaymentInstructionPrintResult
// ---------------------------------------------------------------------------

/// Print result query request data (max 5 items per call).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintResultQueryRequestData {
    /// Customer tax ID (8-10 digits).
    #[serde(rename = "payerIxd")]
    pub payer_ixd: String,
    /// ERP transaction sequence (max 22).
    #[serde(rename = "txSeq")]
    pub tx_seq: String,
}

/// Full print result query request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintResultQueryRequest {
    #[serde(rename = "rqHeader")]
    pub rq_header: RqHeader,
    #[serde(rename = "rqData")]
    pub rq_data: PrintResultQueryRequestData,
}

/// Print result entry (PDF binary as Base64).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrintResultEntry {
    /// Payer tax ID (15 chars).
    #[serde(rename = "payerIxd", default)]
    pub payer_ixd: String,
    /// ERP transaction sequence (22 chars).
    #[serde(rename = "txSeq", default)]
    pub tx_seq: String,
    /// PDF binary, Base64-encoded.
    pub content: String,
}

/// Print result response data.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PrintResultResponseData {
    /// Print result entries (max 5 per call).
    #[serde(rename = "printResultList", default)]
    pub print_result_list: Vec<PrintResultEntry>,
}

// ---------------------------------------------------------------------------
// Currency Codes
// ---------------------------------------------------------------------------

/// Supported currency codes (from E.Sun appendix).
pub const CURRENCY_CODES: &[&str] = &[
    "USD", "CNY", "HKD", "JPY", "EUR", "AUD", "CAD", "GBP", "ZAR", "NZD", "CHF", "SEK", "SGD",
    "MXN", "THB",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_result_success() {
        assert!(is_api_success("910000"));
        assert!(!is_api_success("910071"));
        assert!(!is_api_success("999999"));
    }

    #[test]
    fn test_tx_status_success() {
        assert!(is_tx_success("0000"));
        assert!(!is_tx_success("4000"));
    }

    #[test]
    fn test_tx_status_pending() {
        assert!(is_tx_pending("1000"));
        assert!(is_tx_pending("1008"));
        assert!(!is_tx_pending("1009"));
        assert!(!is_tx_pending("0000"));
    }

    #[test]
    fn test_tx_status_failed() {
        assert!(is_tx_failed("4000"));
        assert!(!is_tx_failed("0000"));
    }

    #[test]
    fn test_api_response_success_check() {
        let resp = ApiResponse::<TwdPaymentResponseData> {
            rs_header: RsHeader {
                msg_id: "1234567890202209150000001".to_string(),
                log_id: "LOG123".to_string(),
                response_time: "2022/09/15 10:00:00".to_string(),
            },
            rs_status: RsStatus {
                code: "910000".to_string(),
                message: "Success".to_string(),
                more_info: String::new(),
            },
            rs_data: Some(TwdPaymentResponseData::default()),
        };
        assert!(resp.is_success());
    }

    #[test]
    fn test_api_response_failure_no_data() {
        let resp = ApiResponse::<TwdPaymentResponseData> {
            rs_header: RsHeader {
                msg_id: "1234567890202209150000001".to_string(),
                log_id: "LOG123".to_string(),
                response_time: "2022/09/15 10:00:00".to_string(),
            },
            rs_status: RsStatus {
                code: "910071".to_string(),
                message: "Field error".to_string(),
                more_info: "TX001|amount invalid".to_string(),
            },
            rs_data: None,
        };
        assert!(!resp.is_success());
        assert_eq!(resp.rs_status.more_info, "TX001|amount invalid");
    }

    #[test]
    fn test_twd_payment_tx_serde_roundtrip() {
        let tx = TwdPaymentTx {
            payer_account_no: "1234567890123".to_string(),
            payer_id: "1234567890".to_string(),
            payee_bank_id: "013".to_string(),
            payee_branch_id: "0001".to_string(),
            payee_account_no: "9876543210".to_string(),
            payee_name: "測試收款人".to_string(),
            payee_id: String::new(),
            tx_date: "20261103".to_string(),
            tx_amt: "100000".to_string(),
            charge_fee_type: "0".to_string(),
            memo: "測試付款".to_string(),
            notify_method: "1".to_string(),
            notify_email: String::new(),
            notify_fax: String::new(),
            write_off_no: String::new(),
            remark: String::new(),
            tx_seq: "IOB20261102001-0001".to_string(),
            detail_list: vec![],
        };
        let json = serde_json::to_string(&tx).unwrap();
        let parsed: TwdPaymentTx = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.payer_account_no, "1234567890123");
        assert_eq!(parsed.tx_seq, "IOB20261102001-0001");
    }

    #[test]
    fn test_currency_codes_count() {
        assert_eq!(CURRENCY_CODES.len(), 15);
        assert!(CURRENCY_CODES.contains(&"USD"));
        assert!(!CURRENCY_CODES.contains(&"TWD"));
    }

    #[test]
    fn test_statement_request_serde_roundtrip() {
        let req = StatementQueryRequest {
            rq_header: RqHeader {
                msg_id: "12345678202611060000001".to_string(),
                request_time: "2026/11/06 10:00:00".to_string(),
                customer_ixd: "12345678".to_string(),
                source_system: "ERP-BANKPAY".to_string(),
            },
            rq_data: StatementQueryRequestData {
                company_id: "12345678".to_string(),
                demand_dept_acc: "1234567890123".to_string(),
                s_tx_date: "20261101".to_string(),
                e_tx_date: "20261106".to_string(),
                s_tx_time: String::new(),
                e_tx_time: String::new(),
                curr: String::new(),
                db_ckn: String::new(),
                id_no_asset_code: String::new(),
                order_desc_type: "1".to_string(),
                query_start_index: "1".to_string(),
                first_dept_tx_dtl_seq_no: String::new(),
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"demandDeptAcc\""));
        assert!(json.contains("\"queryStartIndex\""));
        let parsed: StatementQueryRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.rq_data.company_id, "12345678");
        assert_eq!(parsed.rq_data.demand_dept_acc, "1234567890123");
    }

    #[test]
    fn test_statement_response_empty_and_max_pagination() {
        // Empty list round-trip
        let data = StatementResponseData {
            dept_tx_dtl_list: vec![],
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: StatementResponseData = serde_json::from_str(&json).unwrap();
        assert!(parsed.dept_tx_dtl_list.is_empty());

        // Max pagination (500 items)
        let mut rows = Vec::with_capacity(500);
        for i in 0..500 {
            rows.push(DeptTxDtl {
                dept_tx_dtl_seq_no: format!("{i:019}"),
                ..Default::default()
            });
        }
        let page = StatementResponseData {
            dept_tx_dtl_list: rows,
        };
        let json = serde_json::to_string(&page).unwrap();
        let parsed: StatementResponseData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.dept_tx_dtl_list.len(), 500);
        assert_eq!(
            parsed.dept_tx_dtl_list[0].dept_tx_dtl_seq_no,
            "0000000000000000000"
        );
        assert_eq!(
            parsed.dept_tx_dtl_list[499].dept_tx_dtl_seq_no,
            "0000000000000000499"
        );
    }

    #[test]
    fn test_fx_request_serde_roundtrip() {
        let fx_tx = ForeignTx {
            tx_seq: "IOB20261102001-0001".to_string(),
            tx_date: "20261103".to_string(),
            payer_id: "1234567890".to_string(),
            payer_account_no: "1234567890123".to_string(),
            payer_currency: "USD".to_string(),
            payer_amount: "1000.00".to_string(),
            payee_account_no: "DE89370400440532013000".to_string(),
            payee_currency: "EUR".to_string(),
            payee_amount: "920.50".to_string(),
            payee_name: "European Payee".to_string(),
            payee_bank_swift_code: "DEUTDEFF".to_string(),
            payee_bank_address_country: "DE".to_string(),
            remittance_type: "1".to_string(),
            fee_type: "0".to_string(),
            fee_debit_account_no: "1234567890123".to_string(),
            fee_debit_currency: "USD".to_string(),
            notification_type: "2".to_string(),
            notification_info: "finance@example.com".to_string(),
        };
        let list_json = serde_json::to_string(&vec![fx_tx.clone()]).unwrap();
        // content is Base64(JSON) in transport; we verify JSON structure here.
        let parsed: Vec<ForeignTx> = serde_json::from_str(&list_json).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].tx_seq, "IOB20261102001-0001");
        assert_eq!(parsed[0].payer_currency, "USD");
        assert_eq!(parsed[0].payee_currency, "EUR");
    }

    #[test]
    fn test_fx_result_entry_serde_roundtrip() {
        let entry = FxResultEntry {
            status: "1031".to_string(),
            status_desc: "電文已發送".to_string(),
            transaction_reference_no: "FX1234567890".to_string(),
            listing_rate: "31.250000".to_string(),
            dealing_rate: "31.200000".to_string(),
            fee_debit_amount: "150.00".to_string(),
            value_date: "20261107".to_string(),
            actual_tx_date: "20261106".to_string(),
            sys_error_desc: String::new(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"transactionReferenceNo\""));
        assert!(json.contains("\"listingRate\""));
        let parsed: FxResultEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.status, "1031");
        assert_eq!(parsed.transaction_reference_no, "FX1234567890");
    }

    #[test]
    fn test_balance_query_serde_roundtrip() {
        let req = BalanceQueryRequest {
            rq_header: RqHeader {
                msg_id: "12345678202611060000001".to_string(),
                request_time: "2026/11/06 10:00:00".to_string(),
                customer_ixd: "12345678".to_string(),
                source_system: "ERP-BANKPAY".to_string(),
            },
            rq_data: BalanceQueryRequestData {
                company_id: "12345678".to_string(),
                curr_type: "1".to_string(),
                demand_dept_acc: String::new(),
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"currType\""));
        let parsed: BalanceQueryRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.rq_data.curr_type, "1");

        let resp = BalanceResponseData {
            dept_blnc_dtl_list: vec![DeptBlncDtl {
                demand_dept_acc: "1234567890123".to_string(),
                curr: "TWD".to_string(),
                acc_type: "1".to_string(),
                act_balance: "1000000.00".to_string(),
                avl_balance: "900000.00".to_string(),
                today_exg_amt: "50000.00".to_string(),
                next_exg_amt: "0.00".to_string(),
            }],
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: BalanceResponseData = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed.dept_blnc_dtl_list[0].demand_dept_acc,
            "1234567890123"
        );
    }

    #[test]
    fn test_print_result_base64_roundtrip() {
        let entry = PrintResultEntry {
            payer_ixd: "12345678".to_string(),
            tx_seq: "IOB20261102001-0001".to_string(),
            // Minimal PDF header + trailer for round-trip testing
            content: "JVBERi0xLjQKJQ==".to_string(), // "%PDF-1.4\n" in base64
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"content\""));
        let parsed: PrintResultEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.payer_ixd, "12345678");
        assert_eq!(parsed.content, "JVBERi0xLjQKJQ==");

        // Max 5 items per call
        let entries: Vec<PrintResultEntry> = (0..5)
            .map(|i| PrintResultEntry {
                payer_ixd: "12345678".to_string(),
                tx_seq: format!("TX-{i:04}"),
                content: format!("CONTENT-{i}"),
            })
            .collect();
        let resp = PrintResultResponseData {
            print_result_list: entries,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: PrintResultResponseData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.print_result_list.len(), 5);
    }
}

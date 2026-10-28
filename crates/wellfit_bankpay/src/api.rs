#![forbid(unsafe_code)]

//! E.Sun Bank ERP integration API models (V2.8).
//!
//! Serde structs for the request/response envelopes used by the
//! `銀企直連安控元件` (security component) on port 8443.
//!
//! Reference: `【玉山銀行】企業ERP銀企直連串接_技術文件V2.8_v1.pdf`

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
}

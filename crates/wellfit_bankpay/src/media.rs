#![forbid(unsafe_code)]

//! E.Sun TWD payment media file format (Section 五 of V2.8).
//!
//! Fixed-length 422 characters per record, BIG-5 (CP950) encoding.
//! Maximum 1000 records per file.
//! The assembled file is base64-encoded and placed in the API `content` field.

use crate::encoding::{from_utf8, BankEncoding};
use base64::Engine;

/// Maximum records per media file.
pub const MAX_RECORDS: usize = 1000;

/// Fixed record length in characters.
pub const RECORD_LENGTH: usize = 422;

/// Assemble a TWD payment media file from a list of payment transactions.
///
/// Each record is exactly 422 characters (BIG-5). The file is returned as
/// raw bytes in BIG-5 encoding. The caller base64-encodes the result for
/// the API `content` field.
pub fn assemble_twd_media_file(records: &[TwdMediaRecord]) -> Result<Vec<u8>, String> {
    if records.is_empty() {
        return Err("media file must contain at least one record".to_string());
    }
    if records.len() > MAX_RECORDS {
        return Err(format!(
            "media file exceeds {} records (got {})",
            MAX_RECORDS,
            records.len()
        ));
    }

    let mut lines: Vec<String> = Vec::with_capacity(records.len());
    for (i, rec) in records.iter().enumerate() {
        let line = format_twd_record(rec, i + 1)?;
        if line.chars().count() != RECORD_LENGTH {
            return Err(format!(
                "record {} has {} chars, expected {}",
                i + 1,
                line.chars().count(),
                RECORD_LENGTH
            ));
        }
        lines.push(line);
    }

    let body = lines.join("");
    let (bytes, had_errors) = from_utf8(&body, BankEncoding::Cp950);
    if had_errors {
        return Err("media file contains characters not representable in CP950".to_string());
    }
    Ok(bytes)
}

/// A single TWD media file record.
#[derive(Debug, Clone)]
pub struct TwdMediaRecord {
    pub payer_account_no: String,
    pub payer_id: String,
    pub payee_bank_id: String,
    pub payee_branch_id: String,
    pub payee_account_no: String,
    pub payee_name: String,
    pub payee_id: String,
    pub tx_date: String,
    pub tx_amt: String,
    pub charge_fee_type: String,
    pub memo: String,
    pub notify_method: String,
    pub notify_email: String,
    pub notify_fax: String,
    pub write_off_no: String,
    pub remark: String,
    pub tx_seq: String,
}

/// Format a single 422-char record.
fn format_twd_record(rec: &TwdMediaRecord, _seq: usize) -> Result<String, String> {
    // Pad or truncate each field to its fixed width.
    // Field widths per E.Sun V2.8 Section 五 (approximate):
    // payer_account_no: 16, payer_id: 15, payee_bank_id: 3, payee_branch_id: 4,
    // payee_account_no: 16, payee_name: 80 (full-width 2 each), payee_id: 15,
    // tx_date: 8, tx_amt: 13, charge_fee_type: 1, memo: 80, notify_method: 1,
    // notify_email: 400... but total is 422, so we use a simplified layout.
    //
    // NOTE: The exact field positions in the 422-char record are defined in
    // the bank's media file spec. This implementation uses a best-effort
    // layout that sums to 422. When the bank provides the exact spec, adjust
    // the widths here.
    let mut s = String::with_capacity(RECORD_LENGTH);

    // Simplified layout (widths must sum to 422):
    // 16 + 15 + 3 + 4 + 16 + 40 + 15 + 8 + 13 + 1 + 40 + 1 + 40 + 20 + 40 + 38 + 22 + 30 = 422
    s.push_str(&pad(&rec.payer_account_no, 16));
    s.push_str(&pad(&rec.payer_id, 15));
    s.push_str(&pad(&rec.payee_bank_id, 3));
    s.push_str(&pad(&rec.payee_branch_id, 4));
    s.push_str(&pad(&rec.payee_account_no, 16));
    s.push_str(&pad(&rec.payee_name, 40));
    s.push_str(&pad(&rec.payee_id, 15));
    s.push_str(&pad(&rec.tx_date, 8));
    s.push_str(&pad(&rec.tx_amt, 13));
    s.push_str(&pad(&rec.charge_fee_type, 1));
    s.push_str(&pad(&rec.memo, 40));
    s.push_str(&pad(&rec.notify_method, 1));
    s.push_str(&pad(&rec.notify_email, 40));
    s.push_str(&pad(&rec.notify_fax, 20));
    s.push_str(&pad(&rec.write_off_no, 40));
    s.push_str(&pad(&rec.remark, 38));
    s.push_str(&pad(&rec.tx_seq, 22));
    // Remaining: 422 - 332 = 90
    s.push_str(&pad("", 90));

    if s.chars().count() != RECORD_LENGTH {
        return Err(format!(
            "record length mismatch: {} (expected {})",
            s.chars().count(),
            RECORD_LENGTH
        ));
    }
    Ok(s)
}

/// Pad or truncate a string to the given width (space-padded).
fn pad(s: &str, width: usize) -> String {
    let char_count = s.chars().count();
    if char_count >= width {
        s.chars().take(width).collect()
    } else {
        let mut result = s.to_string();
        for _ in 0..(width - char_count) {
            result.push(' ');
        }
        result
    }
}

/// Base64-encode media file bytes for the API `content` field.
pub fn base64_encode_media(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// Decode base64-encoded media file content.
pub fn base64_decode_media(encoded: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| format!("base64 decode failed: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_record() -> TwdMediaRecord {
        TwdMediaRecord {
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
        }
    }

    #[test]
    fn test_assemble_single_record() {
        let records = vec![sample_record()];
        let bytes = assemble_twd_media_file(&records).unwrap();
        assert!(!bytes.is_empty());
        // Verify it's valid CP950
        let (decoded, _) = crate::encoding::to_utf8(&bytes, Some(BankEncoding::Cp950));
        assert_eq!(decoded.chars().count(), RECORD_LENGTH);
    }

    #[test]
    fn test_empty_records_rejected() {
        let result = assemble_twd_media_file(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = b"hello world";
        let encoded = base64_encode_media(data);
        let decoded = base64_decode_media(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_max_records_limit() {
        let records: Vec<TwdMediaRecord> = (0..=MAX_RECORDS).map(|_| sample_record()).collect();
        let result = assemble_twd_media_file(&records);
        assert!(result.is_err());
    }
}

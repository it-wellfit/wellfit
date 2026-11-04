#![forbid(unsafe_code)]

//! Shadow Capture — read-only verification evidence for a vendor export.
//!
//! Implements `docs/operations/SHADOW_CAPTURE_SPECIFICATION.md` (S1-1):
//! - Read-only: the input file is only read; artifacts are written under the
//!   caller-provided artifact root and nowhere else.
//! - 100 MiB single-file cap, consistent with the workspace ingestion limit.
//! - Column contract resolved by name before any row is read (no positional
//!   fallback).
//! - Independent recomputation of the Fuyi inventory identity
//!   (End = Begin + In - Out + Adj, overflow-safe). The vendor's stated values
//!   and our recomputed values are hashed separately; the capture matches only
//!   when every row agrees.
//! - Evidence JSON per capture, schema `wellfit-shadow-evidence/v1`, written
//!   to `<artifact_root>/shadow-mode/`.

use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use wellfit_core::text::{read_capped, MAX_INPUT_BYTES};
use wellfit_reports::report_automator::fuyi_inventory_report_contract;

/// Evidence schema identifier written into every capture document.
pub const SHADOW_EVIDENCE_SCHEMA: &str = "wellfit-shadow-evidence/v1";

/// Errors raised by the shadow capture.
#[derive(Debug)]
pub enum CaptureError {
    /// The input could not be read (missing, oversized, or not UTF-8).
    Input(String),
    /// The input header does not satisfy the pipeline contract.
    Contract(String),
    /// The evidence document could not be written.
    Evidence(String),
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureError::Input(msg) => write!(f, "input error: {msg}"),
            CaptureError::Contract(msg) => write!(f, "contract error: {msg}"),
            CaptureError::Evidence(msg) => write!(f, "evidence error: {msg}"),
        }
    }
}

impl std::error::Error for CaptureError {}

/// One row whose vendor-stated closing balance differs from our recomputation,
/// or that could not be verified at all.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct MismatchRow {
    /// One-based line number in the source file (header included).
    pub line: usize,
    /// ERP item number as written in the report (empty when unreadable).
    pub item_no: String,
    /// Warehouse code as written in the report (empty when unreadable).
    pub warehouse_code: String,
    /// Closing balance stated by the vendor, when parsable.
    pub vendor_end_inv: Option<i64>,
    /// Closing balance recomputed by us, when computable.
    pub our_expected_end: Option<i64>,
    /// Why the row is listed: `value mismatch`, `overflow`, or `unparsable`.
    pub note: String,
}

/// The evidence document for one shadow capture.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ShadowEvidence {
    /// Schema identifier; see [`SHADOW_EVIDENCE_SCHEMA`].
    pub schema: String,
    /// Capture time, Unix seconds.
    pub captured_at_unix: u64,
    /// Pipeline that was captured (`reports`).
    pub pipeline: String,
    /// Batch identifier supplied by the operator.
    pub batch_id: String,
    /// Path of the captured vendor export.
    pub input_path: String,
    /// SHA-256 of the input bytes.
    pub input_sha256: String,
    /// Input size in bytes.
    pub input_bytes: usize,
    /// Number of data rows seen.
    pub records_total: usize,
    /// Number of rows whose values matched our recomputation.
    pub records_verified: usize,
    /// Number of rows listed as mismatches.
    pub records_mismatched: usize,
    /// True when every row agreed (digests equal and no listed mismatches).
    pub row_level_match: bool,
    /// SHA-256 over the canonical vendor value lines.
    pub vendor_digest: String,
    /// SHA-256 over the canonical recomputed value lines.
    pub our_digest: String,
    /// The mismatched rows (full list; callers may cap display).
    pub mismatches: Vec<MismatchRow>,
}

/// A completed capture: the evidence plus where it was written.
#[derive(Debug, Clone)]
pub struct CaptureOutcome {
    /// The evidence document.
    pub evidence: ShadowEvidence,
    /// Absolute or relative path of the written evidence JSON.
    pub evidence_path: PathBuf,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn unix_now() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Keep filename-safe characters only; everything else becomes `-`.
fn sanitize_token(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// Capture a Fuyi ERP inventory report export and verify every row against
/// the inventory identity `End = Begin + In - Out + Adj`.
///
/// Read-only with respect to `input`; the only write is the evidence JSON
/// under `artifact_root/shadow-mode/`.
pub fn capture_reports_export(
    input: &Path,
    artifact_root: &Path,
    batch_id: &str,
) -> Result<CaptureOutcome, CaptureError> {
    // Read with the workspace-wide 100 MiB cap (read-only).
    let content = read_capped(input, MAX_INPUT_BYTES).map_err(CaptureError::Input)?;
    let input_sha256 = sha256_hex(content.as_bytes());
    let input_bytes = content.len();

    // Tolerate a UTF-8 BOM so a report exported with one still resolves.
    let body = content.strip_prefix('\u{feff}').unwrap_or(&content);

    let contract = fuyi_inventory_report_contract();
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(body.as_bytes());
    let headers = reader
        .headers()
        .map_err(|e| CaptureError::Contract(e.to_string()))?
        .clone();
    let header_refs: Vec<&str> = headers.iter().collect();
    let map = contract
        .resolve(&header_refs)
        .map_err(CaptureError::Contract)?;

    let index = |name: &str| map.index(name).map_err(CaptureError::Contract);
    let idx_item = index("ITEM_NO")?;
    let idx_name = index("ITEM_NAME")?;
    let idx_wh = index("WAREHOUSE_CODE")?;
    let idx_begin = index("BEGIN_INV")?;
    let idx_in = index("IN_QTY")?;
    let idx_out = index("OUT_QTY")?;
    let idx_adj = index("ADJ_QTY")?;
    let idx_end = index("END_INV")?;
    let _ = idx_name; // item name is captured by the engine, not needed for the identity

    let mut vendor_canonical = String::new();
    let mut our_canonical = String::new();
    let mut mismatches: Vec<MismatchRow> = Vec::new();
    let mut records_total: usize = 0;
    let mut records_verified: usize = 0;

    for (i, record) in reader.records().enumerate() {
        let line = i + 2;
        records_total += 1;

        let record = match record {
            Ok(record) => record,
            Err(e) => {
                mismatches.push(MismatchRow {
                    line,
                    item_no: String::new(),
                    warehouse_code: String::new(),
                    vendor_end_inv: None,
                    our_expected_end: None,
                    note: format!("unparsable: {e}"),
                });
                vendor_canonical.push_str("?|?|?\n");
                our_canonical.push_str("?|?|unparsable\n");
                continue;
            }
        };

        let item_no = record.get(idx_item).unwrap_or("").trim().to_string();
        let warehouse_code = record.get(idx_wh).unwrap_or("").trim().to_string();

        let parse_cell = |idx: usize| -> Result<i64, String> {
            let raw = record.get(idx).unwrap_or("").trim();
            raw.parse::<i64>()
                .map_err(|_| format!("cell '{}' is not an integer", raw))
        };

        let parsed = (
            parse_cell(idx_begin),
            parse_cell(idx_in),
            parse_cell(idx_out),
            parse_cell(idx_adj),
            parse_cell(idx_end),
        );

        let (vendor_end, our_expected, note) = match parsed {
            (Ok(begin), Ok(in_qty), Ok(out_qty), Ok(adj_qty), Ok(end)) => {
                let expected = begin
                    .checked_add(in_qty)
                    .and_then(|v| v.checked_sub(out_qty))
                    .and_then(|v| v.checked_add(adj_qty));
                match expected {
                    Some(expected) if expected == end => (Some(end), Some(expected), None),
                    Some(expected) => (
                        Some(end),
                        Some(expected),
                        Some("value mismatch".to_string()),
                    ),
                    None => (Some(end), None, Some("overflow".to_string())),
                }
            }
            (b, i2, o, a, e) => {
                let which = [
                    ("BEGIN_INV", b),
                    ("IN_QTY", i2),
                    ("OUT_QTY", o),
                    ("ADJ_QTY", a),
                    ("END_INV", e),
                ]
                .into_iter()
                .find_map(|(name, cell)| cell.err().map(|err| format!("{name}: {err}")));
                (
                    None,
                    None,
                    Some(format!(
                        "unparsable: {}",
                        which.unwrap_or_else(|| "unknown cell".to_string())
                    )),
                )
            }
        };

        vendor_canonical.push_str(&format!(
            "{}|{}|{}\n",
            item_no,
            warehouse_code,
            vendor_end
                .map(|v| v.to_string())
                .unwrap_or_else(|| "?".to_string())
        ));
        our_canonical.push_str(&format!(
            "{}|{}|{}\n",
            item_no,
            warehouse_code,
            our_expected
                .map(|v| v.to_string())
                .unwrap_or_else(|| "?".to_string())
        ));

        match note {
            None => records_verified += 1,
            Some(note) => mismatches.push(MismatchRow {
                line,
                item_no,
                warehouse_code,
                vendor_end_inv: vendor_end,
                our_expected_end: our_expected,
                note,
            }),
        }
    }

    let vendor_digest = sha256_hex(vendor_canonical.as_bytes());
    let our_digest = sha256_hex(our_canonical.as_bytes());
    let row_level_match = vendor_digest == our_digest && mismatches.is_empty();

    let evidence = ShadowEvidence {
        schema: SHADOW_EVIDENCE_SCHEMA.to_string(),
        captured_at_unix: unix_now(),
        pipeline: "reports".to_string(),
        batch_id: batch_id.to_string(),
        input_path: input.display().to_string(),
        input_sha256,
        input_bytes,
        records_total,
        records_verified,
        records_mismatched: mismatches.len(),
        row_level_match,
        vendor_digest,
        our_digest,
        mismatches,
    };

    let dir = artifact_root.join("shadow-mode");
    fs::create_dir_all(&dir).map_err(|e| CaptureError::Evidence(e.to_string()))?;
    let filename = format!(
        "{}-{}.json",
        evidence.captured_at_unix,
        sanitize_token(batch_id)
    );
    let evidence_path = dir.join(filename);
    let json = serde_json::to_string_pretty(&evidence)
        .map_err(|e| CaptureError::Evidence(e.to_string()))?;
    fs::write(&evidence_path, json).map_err(|e| CaptureError::Evidence(e.to_string()))?;

    Ok(CaptureOutcome {
        evidence,
        evidence_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("wellfit-shadow-capture-{name}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    fn write_input(root: &Path, csv: &str) -> PathBuf {
        let path = root.join("vendor-export.csv");
        fs::write(&path, csv).expect("write input");
        path
    }

    const BALANCED: &str = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\n8,Top Black,WH-A,500,200,50,0,650\n16,Top Navy,WH-A,300,100,20,-10,370\n";

    #[test]
    fn test_capture_all_balanced_matches() {
        let root = temp_root("balanced");
        let input = write_input(&root, BALANCED);
        let outcome =
            capture_reports_export(&input, &root.join("artifacts"), "BATCH-001").expect("capture");

        assert!(outcome.evidence.row_level_match);
        assert_eq!(outcome.evidence.records_total, 2);
        assert_eq!(outcome.evidence.records_verified, 2);
        assert_eq!(outcome.evidence.records_mismatched, 0);
        assert!(outcome.evidence.mismatches.is_empty());
        assert_eq!(outcome.evidence.vendor_digest, outcome.evidence.our_digest);
        assert_eq!(outcome.evidence.schema, SHADOW_EVIDENCE_SCHEMA);
        assert_eq!(outcome.evidence.pipeline, "reports");
        assert!(outcome.evidence_path.exists());
        assert!(outcome
            .evidence_path
            .to_string_lossy()
            .contains("BATCH-001"));
    }

    #[test]
    fn test_capture_detects_value_mismatch() {
        let root = temp_root("mismatch");
        let csv = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\n8,Top Black,WH-A,500,200,50,0,999\n";
        let input = write_input(&root, csv);
        let outcome =
            capture_reports_export(&input, &root.join("artifacts"), "BATCH-002").expect("capture");

        assert!(!outcome.evidence.row_level_match);
        assert_eq!(outcome.evidence.records_mismatched, 1);
        let m = &outcome.evidence.mismatches[0];
        assert_eq!(m.line, 2);
        assert_eq!(m.item_no, "8");
        assert_eq!(m.vendor_end_inv, Some(999));
        assert_eq!(m.our_expected_end, Some(650));
        assert_eq!(m.note, "value mismatch");
        assert_ne!(outcome.evidence.vendor_digest, outcome.evidence.our_digest);
    }

    #[test]
    fn test_capture_flags_overflow() {
        let root = temp_root("overflow");
        let csv = format!(
            "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\n8,Top Black,WH-A,{max},1,0,0,0\n",
            max = i64::MAX
        );
        let input = write_input(&root, &csv);
        let outcome =
            capture_reports_export(&input, &root.join("artifacts"), "BATCH-003").expect("capture");

        assert!(!outcome.evidence.row_level_match);
        let m = &outcome.evidence.mismatches[0];
        assert_eq!(m.note, "overflow");
        assert_eq!(m.our_expected_end, None);
    }

    #[test]
    fn test_capture_flags_unparsable_cell() {
        let root = temp_root("unparsable");
        let csv = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,ADJ_QTY,END_INV\n8,Top Black,WH-A,500,abc,50,0,650\n";
        let input = write_input(&root, csv);
        let outcome =
            capture_reports_export(&input, &root.join("artifacts"), "BATCH-004").expect("capture");

        assert!(!outcome.evidence.row_level_match);
        let m = &outcome.evidence.mismatches[0];
        assert!(m.note.contains("IN_QTY"));
        assert_eq!(m.vendor_end_inv, None);
    }

    #[test]
    fn test_capture_missing_column_fails_before_rows() {
        let root = temp_root("missing");
        let csv = "ITEM_NO,ITEM_NAME,WAREHOUSE_CODE,BEGIN_INV,IN_QTY,OUT_QTY,END_INV\n8,Top Black,WH-A,500,200,50,650\n";
        let input = write_input(&root, csv);
        let err = capture_reports_export(&input, &root.join("artifacts"), "BATCH-005")
            .expect_err("must fail");
        assert!(matches!(err, CaptureError::Contract(_)));
        assert!(!root.join("artifacts").exists());
    }

    #[test]
    fn test_capture_reads_reordered_header_by_name() {
        let root = temp_root("reordered");
        let csv = "END_INV,ADJ_QTY,OUT_QTY,IN_QTY,BEGIN_INV,WAREHOUSE_CODE,ITEM_NAME,ITEM_NO\n650,0,50,200,500,WH-A,Top Black,8\n";
        let input = write_input(&root, csv);
        let outcome =
            capture_reports_export(&input, &root.join("artifacts"), "BATCH-006").expect("capture");
        assert!(outcome.evidence.row_level_match);
    }

    #[test]
    fn test_capture_tolerates_utf8_bom() {
        let root = temp_root("bom");
        let csv = format!("\u{feff}{BALANCED}");
        let input = write_input(&root, &csv);
        let outcome =
            capture_reports_export(&input, &root.join("artifacts"), "BATCH-007").expect("capture");
        assert!(outcome.evidence.row_level_match);
    }

    #[test]
    fn test_capture_missing_input_fails() {
        let root = temp_root("missing-input");
        let err = capture_reports_export(
            &root.join("does-not-exist.csv"),
            &root.join("artifacts"),
            "BATCH-008",
        )
        .expect_err("must fail");
        assert!(matches!(err, CaptureError::Input(_)));
    }
}

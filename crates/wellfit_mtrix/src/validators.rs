#![forbid(unsafe_code)]

//! Wellfit MTRIX - Validation Rule Engine (VR-01 .. VR-12)
//!
//! Implements the validation rules declared in the MTRIX field dictionary
//! (L2 "validation rules" sheet) together with the original manual page
//! references. Each finding carries the rule, its severity, the action taken
//! and the exact key that caused it, so the audit trail is reconstructable.
//!
//! Severity levels follow the internal two-tier model (MTRIX-016):
//! - `Error`   - the row or the file cannot be accepted.
//! - `Warning` - the row is accepted, but a data-quality caveat is recorded.
//! - `Info`    - an observation that needs no action.
//!
//! Mandatory-ness is not defined by the manual (MTRIX-003). An empty value in a
//! field whose mandatory-ness is unknown is therefore a `Warning`, never a
//! rejection: the row is kept and the caveat is reported.

use serde::{Deserialize, Serialize};

/// A validation rule from the MTRIX rule set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VrRule {
    /// File name prefix, timestamp and delimiter.
    #[serde(rename = "VR-01")]
    Vr01,
    /// Header names, order and column count.
    #[serde(rename = "VR-02")]
    Vr02,
    /// Date format and calendar validity.
    #[serde(rename = "VR-03")]
    Vr03,
    /// Sales document type value set.
    #[serde(rename = "VR-04")]
    Vr04,
    /// Duplicate sales transaction.
    #[serde(rename = "VR-05")]
    Vr05,
    /// Negative inventory quantity.
    #[serde(rename = "VR-06")]
    Vr06,
    /// Product coded field value sets.
    #[serde(rename = "VR-07")]
    Vr07,
    /// Cross-file: sales customer exists in the customer master.
    #[serde(rename = "VR-08")]
    Vr08,
    /// Cross-file: sales item exists in inventory and product master.
    #[serde(rename = "VR-09")]
    Vr09,
    /// Cross-file: salesperson exists in the sales force master for the same customer.
    #[serde(rename = "VR-10")]
    Vr10,
    /// Cross-file: inventory item exists in the product master.
    #[serde(rename = "VR-11")]
    Vr11,
    /// Acceptance: record counts reconciled against the management report.
    #[serde(rename = "VR-12")]
    Vr12,
    /// Water-level alignment: the inventory snapshot belongs to the same
    /// movement period as the sales file (original manual p.18).
    #[serde(rename = "VR-13")]
    Vr13,
    /// Cross-file: sales force customer exists in the customer master
    /// (original manual p.34).
    #[serde(rename = "VR-14")]
    Vr14,
    /// Field format validation: the per-field specific validations declared by
    /// the field dictionary (ISO 3166-1 alpha-3 country code, EAN13 check
    /// digit, postal code format) and the column shift they expose.
    #[serde(rename = "VR-15")]
    Vr15,
}

impl VrRule {
    /// Rule identifier, e.g. `VR-08`.
    pub fn id(self) -> &'static str {
        match self {
            VrRule::Vr01 => "VR-01",
            VrRule::Vr02 => "VR-02",
            VrRule::Vr03 => "VR-03",
            VrRule::Vr04 => "VR-04",
            VrRule::Vr05 => "VR-05",
            VrRule::Vr06 => "VR-06",
            VrRule::Vr07 => "VR-07",
            VrRule::Vr08 => "VR-08",
            VrRule::Vr09 => "VR-09",
            VrRule::Vr10 => "VR-10",
            VrRule::Vr11 => "VR-11",
            VrRule::Vr12 => "VR-12",
            VrRule::Vr13 => "VR-13",
            VrRule::Vr14 => "VR-14",
            VrRule::Vr15 => "VR-15",
        }
    }

    /// Scope of the rule, e.g. `Sales`.
    pub fn scope(self) -> &'static str {
        match self {
            VrRule::Vr01 | VrRule::Vr02 => "All files",
            VrRule::Vr03 => "Dates",
            VrRule::Vr04 | VrRule::Vr05 => "Sales",
            VrRule::Vr06 => "Inventory",
            VrRule::Vr07 => "Product",
            VrRule::Vr08 | VrRule::Vr09 | VrRule::Vr10 => "Cross-file",
            VrRule::Vr11 => "Cross-file",
            VrRule::Vr12 => "Acceptance",
            VrRule::Vr13 => "Cross-file",
            VrRule::Vr14 => "Cross-file",
            VrRule::Vr15 => "Field format",
        }
    }

    /// The declared level of a rule.
    ///
    /// `Acceptance` is a reconciliation duty, `Warning` marks a rule that
    /// reports a caveat without rejecting anything.
    pub fn level(self) -> &'static str {
        match self {
            VrRule::Vr12 => "Acceptance",
            VrRule::Vr13 => "Warning",
            _ => "Error",
        }
    }

    /// Every rule, in catalogue order.
    ///
    /// VR-01 .. VR-12 come from the internal field dictionary (L2) rule sheet.
    /// VR-13 and VR-14 are derived from the original manual and are not
    /// present in L2. VR-15 implements the per-field specific validations that
    /// the same dictionary declares column by column.
    pub fn all() -> [VrRule; 15] {
        [
            VrRule::Vr01,
            VrRule::Vr02,
            VrRule::Vr03,
            VrRule::Vr04,
            VrRule::Vr05,
            VrRule::Vr06,
            VrRule::Vr07,
            VrRule::Vr08,
            VrRule::Vr09,
            VrRule::Vr10,
            VrRule::Vr11,
            VrRule::Vr12,
            VrRule::Vr13,
            VrRule::Vr14,
            VrRule::Vr15,
        ]
    }

    /// Number of rules declared by the internal field dictionary.
    pub const L2_DECLARED: usize = 13;

    /// Number of rules derived from the original manual but absent from L2.
    pub const MANUAL_DERIVED: usize = 2;
}

/// How serious a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Cannot be accepted.
    Error,
    /// Accepted with a recorded caveat.
    Warning,
    /// Observation only.
    Info,
}

impl Severity {
    /// Lower-case label used in reports.
    pub fn label(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
        }
    }
}

/// What the engine did about a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    /// The whole file was rejected; none of its rows were accepted.
    FileRejected,
    /// The row was moved to the quarantine ledger.
    RowQuarantined,
    /// The value was repaired; the row was accepted.
    Repaired,
    /// A cross-file referential finding: the offending key is listed, the row
    /// itself is retained. The manual asks to list the key, not to drop the row.
    Listed,
    /// The rule did fail, but the row is kept because the mandatory-ness of the
    /// field is undefined by the manual (MTRIX-003): the caveat is reported
    /// instead of the row being rejected.
    Suppressed,
    /// The rule could not be evaluated because a dependency was missing.
    NotEvaluable,
}

impl Outcome {
    /// Lower-case label used in reports.
    pub fn label(self) -> &'static str {
        match self {
            Outcome::FileRejected => "file-rejected",
            Outcome::RowQuarantined => "row-quarantined",
            Outcome::Repaired => "repaired",
            Outcome::Listed => "listed",
            Outcome::Suppressed => "suppressed",
            Outcome::NotEvaluable => "not-evaluable",
        }
    }
}

/// A single validation finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Rule that produced the finding.
    pub rule: VrRule,
    /// Severity of the finding.
    pub severity: Severity,
    /// Action taken.
    pub outcome: Outcome,
    /// File prefix the finding belongs to.
    pub file: String,
    /// 1-based line number (0 for file-level findings).
    pub line: usize,
    /// The key or value that caused the finding.
    pub key: String,
    /// Human readable explanation.
    pub reason: String,
}

/// Overall acceptance status of a drop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatchStatus {
    /// Every file and every row was accepted.
    Accepted,
    /// At least one file was rejected or at least one row was quarantined.
    PartialAccepted,
    /// Nothing could be accepted.
    Rejected,
}

impl BatchStatus {
    /// Lower-case label used in reports.
    pub fn label(self) -> &'static str {
        match self {
            BatchStatus::Accepted => "accepted",
            BatchStatus::PartialAccepted => "partial-accepted",
            BatchStatus::Rejected => "rejected",
        }
    }
}

/// Per-file processing result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileOutcome {
    /// File prefix.
    pub prefix: String,
    /// File name as delivered.
    pub source_file: String,
    /// SHA-256 of the delivered bytes.
    pub sha256: String,
    /// Whether the file name matched the expected pattern.
    pub filename_ok: bool,
    /// Whether the header matched the specification.
    pub header_ok: bool,
    /// Data rows found in the file.
    pub rows_total: usize,
    /// Rows accepted after validation.
    pub rows_accepted: usize,
    /// Rows moved to the quarantine ledger.
    pub rows_quarantined: usize,
    /// Repairs applied to this file.
    pub repairs_applied: usize,
    /// Whether the whole file was rejected.
    pub rejected: bool,
    /// Reason for a file-level rejection.
    pub reject_reason: String,
}

/// A wording difference between the delivered bytes and the original manual.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deviation {
    /// File prefix.
    pub prefix: String,
    /// Column ordinal.
    pub seq: u8,
    /// Column name as delivered.
    pub source_name: String,
    /// Column name in the original manual.
    pub spec_name: String,
    /// Technical column name used downstream.
    pub canonical_name: String,
    /// Open issue in the MTRIX issue register.
    pub issue_id: String,

}

/// Per-rule counters.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleTally {
    /// Rule identifier.
    pub rule: String,
    /// Findings with `Error` severity.
    pub errors: usize,
    /// Findings with `Warning` severity.
    pub warnings: usize,
    /// Findings with `Info` severity.
    pub info: usize,
    /// Rules that could not be evaluated.
    pub not_evaluable: usize,
}

/// Conformance of the running binary against the declared specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecConformance {
    /// Number of files declared in the specification.
    pub files_declared: usize,
    /// Number of columns declared in the specification.
    pub fields_declared: usize,
    /// Number of columns that carry a provenance record.
    pub fields_with_provenance: usize,
    /// Number of rules declared by the specification.
    pub rules_declared: usize,
    /// Number of rules implemented by the engine.
    pub rules_implemented: usize,
    /// Number of rules coming from the internal field dictionary.
    pub rules_from_field_dictionary: usize,
    /// Number of rules derived from the original manual only.
    pub rules_from_original_manual: usize,
    /// Number of files whose field table has a page reference in the extracted
    /// manual snapshot, so that the citation can be verified mechanically.
    pub files_with_snapshot_page: usize,
}

/// Amounts grouped by country. Currencies are never mixed or converted
/// (MTRIX-023): a country is the reporting unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrencyGroup {
    /// ISO 3166-1 alpha-3 country code.
    pub country_iso: String,
    /// Number of sales rows in the group.
    pub rows: usize,
    /// Sum of quantities, scaled by 10^4.
    pub quantity_units: i64,
    /// Sum of quantity multiplied by unit price, scaled by 10^8.
    pub amount_units: i64,
}

/// The complete validation report for one drop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Report schema version.
    pub schema_version: String,
    /// Batch identifier.
    pub batch_id: String,
    /// Acceptance status.
    pub status: BatchStatus,
    /// UTC timestamp of the report.
    pub generated_at: String,
    /// Per-file results.
    pub files: Vec<FileOutcome>,
    /// All findings.
    pub findings: Vec<Finding>,
    /// Wording deviations against the original manual.
    pub deviations: Vec<Deviation>,
    /// Per-rule counters.
    pub tallies: Vec<RuleTally>,
    /// Conformance of the binary against the declared specification.
    pub spec_conformance: SpecConformance,
    /// Amounts grouped by country.
    pub currency_groups: Vec<CurrencyGroup>,
    /// Number of repairs applied.
    pub repairs_applied: usize,
}

impl ValidationReport {
    /// Create an empty report skeleton for a batch.
    pub fn new(batch_id: &str) -> Self {
        Self {
            schema_version: "1.0".to_string(),
            batch_id: batch_id.to_string(),
            status: BatchStatus::Accepted,
            generated_at: String::new(),
            files: Vec::new(),
            findings: Vec::new(),
            deviations: Vec::new(),
            tallies: VrRule::all()
                .iter()
                .map(|r| RuleTally {
                    rule: r.id().to_string(),
                    ..RuleTally::default()
                })
                .collect(),
            spec_conformance: SpecConformance {
                files_declared: 0,
                fields_declared: 0,
                fields_with_provenance: 0,
                rules_declared: VrRule::all().len(),
                rules_implemented: VrRule::all().len(),
                rules_from_field_dictionary: VrRule::L2_DECLARED,
                rules_from_original_manual: VrRule::MANUAL_DERIVED,
                files_with_snapshot_page: crate::spec::MANUAL_PDF_PAGE.len(),
            },
            currency_groups: Vec::new(),
            repairs_applied: 0,
        }
    }

    /// Record a finding and update the per-rule counters.
    pub fn add_finding(&mut self, finding: Finding) {
        if let Some(tally) = self
            .tallies
            .iter_mut()
            .find(|t| t.rule == finding.rule.id())
        {
            match finding.severity {
                Severity::Error => tally.errors += 1,
                Severity::Warning => tally.warnings += 1,
                Severity::Info => tally.info += 1,
            }
            if finding.outcome == Outcome::NotEvaluable {
                tally.not_evaluable += 1;
            }
        }
        self.findings.push(finding);
    }

    /// Number of findings with `Error` severity.
    pub fn error_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .count()
    }

    /// Number of findings with `Warning` severity.
    pub fn warning_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Warning)
            .count()
    }

    /// Recompute the acceptance status from the recorded outcomes.
    ///
    /// A drop is `accepted` only when nothing was rejected, no row was
    /// isolated and no validation error was recorded. Cross-file findings are
    /// errors, so a drop that merely *lists* offending keys is
    /// `partial-accepted` and the release gate stays closed.
    pub fn recompute_status(&mut self) {
        let any_rejected = self.files.iter().any(|f| f.rejected);
        let any_quarantined = self.files.iter().any(|f| f.rows_quarantined > 0);
        let any_accepted = self.files.iter().any(|f| f.rows_accepted > 0);
        let any_error = self.error_count() > 0;
        self.status = if !any_accepted && (any_rejected || any_error) {
            BatchStatus::Rejected
        } else if any_rejected || any_quarantined || any_error {
            BatchStatus::PartialAccepted
        } else {
            BatchStatus::Accepted
        };
    }
}

/// Result of checking a delivered file name against VR-01.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilenameCheck {
    /// Whether the prefix and timestamp pattern matched.
    pub pattern_ok: bool,
    /// Whether the embedded date and time are valid.
    pub timestamp_valid: bool,
    /// Non-fatal observations, e.g. a download suffix.
    pub warnings: Vec<String>,
}

/// Expected file name pattern: prefix + DDMMYYYY + HHMMSSmmm.
///
/// The counterpart system appends download residue such as ` 2 (3)`; that part
/// is reported as a warning and never causes a rejection.
pub const FILENAME_PATTERN_LEN: usize = 23;

/// Check a delivered file name against VR-01.
pub fn check_filename(file_name: &str, expected_prefix: &str) -> FilenameCheck {
    let mut warnings = Vec::new();
    let name = file_name.trim();
    if !name.to_uppercase().starts_with(expected_prefix) {
        return FilenameCheck {
            pattern_ok: false,
            timestamp_valid: false,
            warnings,
        };
    }
    if name.len() < FILENAME_PATTERN_LEN {
        return FilenameCheck {
            pattern_ok: false,
            timestamp_valid: false,
            warnings,
        };
    }
    let stamp = &name[expected_prefix.len()..FILENAME_PATTERN_LEN];
    let bytes = stamp.as_bytes();
    let all_digits = bytes.iter().all(|b| b.is_ascii_digit());
    if !all_digits {
        return FilenameCheck {
            pattern_ok: false,
            timestamp_valid: false,
            warnings,
        };
    }
    let day: u32 = stamp[0..2].parse().unwrap_or(0);
    let month: u32 = stamp[2..4].parse().unwrap_or(0);
    let year: u32 = stamp[4..8].parse().unwrap_or(0);
    let hour: u32 = stamp[8..10].parse().unwrap_or(0);
    let minute: u32 = stamp[10..12].parse().unwrap_or(0);
    let second: u32 = stamp[12..14].parse().unwrap_or(0);
    let date_valid = year >= 2000 && (1..=12).contains(&month) && (1..=31).contains(&day);
    let time_valid = hour <= 23 && minute <= 59 && second <= 60;
    if name.len() > FILENAME_PATTERN_LEN {
        warnings.push(format!(
            "file name carries trailing residue '{}' after the timestamp",
            &name[FILENAME_PATTERN_LEN..]
        ));
    }
    FilenameCheck {
        pattern_ok: true,
        timestamp_valid: date_valid && time_valid,
        warnings,
    }
}

/// Check that a `YYYYMMDD` literal is a real calendar date.
pub fn is_valid_yyyymmdd(value: &str) -> bool {
    let clean = value.trim();
    if clean.len() != 8 || !clean.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let year: i32 = clean[0..4].parse().unwrap_or(0);
    let month: u32 = clean[4..6].parse().unwrap_or(0);
    let day: u32 = clean[6..8].parse().unwrap_or(0);
    if !(1..=12).contains(&month) {
        return false;
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        _ => 0,
    };
    day >= 1 && day <= days_in_month
}

/// Compare a delivered header against the specification.
///
/// A column is accepted when its name matches the delivered wording, the manual
/// wording or the technical alias. Anything else is a difference that must be
/// listed and rejected.
pub fn compare_header(spec: &crate::spec::FileSpec, delivered: &[String]) -> (bool, Vec<String>) {
    let mut differences = Vec::new();
    if delivered.len() != spec.column_count() {
        differences.push(format!(
            "column count mismatch: expected {}, delivered {}",
            spec.column_count(),
            delivered.len()
        ));
        return (false, differences);
    }
    for (idx, col) in spec.columns.iter().enumerate() {
        let name = delivered[idx].trim();
        let accepted = [col.source_name, col.spec_name, col.canonical_name];
        if !accepted.iter().any(|a| a.eq_ignore_ascii_case(name)) {
            differences.push(format!(
                "column {} expected one of {:?}, delivered '{}'",
                col.seq, accepted, name
            ));
        }
    }
    (differences.is_empty(), differences)
}

/// Outcome of checking one coded field value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeCheck {
    /// The value is inside the allowed set.
    Allowed,
    /// The value is outside the allowed set.
    NotAllowed,
    /// The value is empty and mandatory-ness is undefined.
    UndefinedMandatory,
}

/// Whether a value is an ISO 3166-1 alpha-3 country code.
///
/// The manual requires the country to be carried as a three letter code
/// ("ISO 3166-1 alpha-3: Three-letter country codes"). The membership of the
/// ISO list is not part of the sources, so only the format is verified.
pub fn is_iso_alpha3(value: &str) -> bool {
    let clean = value.trim();
    clean.len() == 3 && clean.chars().all(|c| c.is_ascii_uppercase())
}

/// Whether a value is a valid EAN13 barcode (13 digits with a valid check digit).
pub fn is_ean13(value: &str) -> bool {
    let clean: String = value
        .trim()
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    if clean.len() != 13 {
        return false;
    }
    let digits: Vec<u32> = clean.chars().map(|c| c.to_digit(10).unwrap_or(0)).collect();
    let sum: u32 = digits[0..12]
        .iter()
        .enumerate()
        .map(|(index, digit)| if index % 2 == 0 { *digit } else { *digit * 3 })
        .sum();
    let check = (10 - (sum % 10)) % 10;
    check == digits[12]
}

/// Check a coded field value against its allowed set.
pub fn check_code(value: &str, allowed: &[&str]) -> CodeCheck {
    let clean = value.trim();
    if clean.is_empty() {
        return CodeCheck::UndefinedMandatory;
    }
    if allowed.iter().any(|a| a.eq_ignore_ascii_case(clean)) {
        CodeCheck::Allowed
    } else {
        CodeCheck::NotAllowed
    }
}


/// Exception notification for cross-file referential findings (VR-08, VR-09).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExceptionNotification {
    pub rule_id: String,
    pub orphan_key: String,
    pub file: String,
    pub line: usize,
    pub suggested_action: String,
    pub notified_at: String,
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{spec_by_role, FileRole};

    #[test]
    fn test_rule_identifiers_are_stable() {
        assert_eq!(VrRule::Vr01.id(), "VR-01");
        assert_eq!(VrRule::Vr08.id(), "VR-08");
        assert_eq!(VrRule::Vr12.id(), "VR-12");
        assert_eq!(VrRule::Vr13.id(), "VR-13");
        assert_eq!(VrRule::Vr14.id(), "VR-14");
        assert_eq!(VrRule::Vr15.id(), "VR-15");
        assert_eq!(VrRule::all().len(), 15);
        assert_eq!(VrRule::L2_DECLARED, 13);
        assert_eq!(VrRule::MANUAL_DERIVED, 2);
        assert_eq!(VrRule::Vr13.level(), "Warning");
        assert_eq!(VrRule::Vr12.level(), "Acceptance");
    }

    #[test]
    fn test_filename_pattern_accepts_delivered_names() {
        let check = check_filename("ALTHCI30062020153005130 2 (3).csv", "ALTHCI");
        assert!(check.pattern_ok);
        assert!(check.timestamp_valid);
        assert_eq!(check.warnings.len(), 1);
    }

    #[test]
    fn test_filename_pattern_rejects_wrong_prefix() {
        let check = check_filename("WRONG30062020153005130.csv", "ALTHCI");
        assert!(!check.pattern_ok);
    }

    #[test]
    fn test_filename_pattern_rejects_impossible_date() {
        let check = check_filename("ALTHCI99132020153005130.csv", "ALTHCI");
        assert!(check.pattern_ok);
        assert!(!check.timestamp_valid);
    }

    #[test]
    fn test_calendar_date_validation() {
        assert!(is_valid_yyyymmdd("20240221"));
        assert!(is_valid_yyyymmdd("20240229"));
        assert!(!is_valid_yyyymmdd("20230229"));
        assert!(!is_valid_yyyymmdd("20241301"));
        assert!(!is_valid_yyyymmdd("2024-02-21"));
        assert!(!is_valid_yyyymmdd(""));
    }

    #[test]
    fn test_header_comparison_accepts_manual_and_alias_wording() {
        let spec = spec_by_role(FileRole::Customer);
        let delivered: Vec<String> = spec
            .columns
            .iter()
            .map(|c| c.source_name.to_string())
            .collect();
        let (ok, diffs) = compare_header(spec, &delivered);
        assert!(ok, "{diffs:?}");

        let mut canonical: Vec<String> = spec
            .columns
            .iter()
            .map(|c| c.canonical_name.to_string())
            .collect();
        let (ok2, _) = compare_header(spec, &canonical);
        assert!(ok2);

        canonical[4] = "STREET".to_string();
        let (ok3, diffs3) = compare_header(spec, &canonical);
        assert!(!ok3);
        assert_eq!(diffs3.len(), 1);
    }

    #[test]
    fn test_code_check_distinguishes_empty_from_invalid() {
        assert_eq!(check_code("N", &["N", "B"]), CodeCheck::Allowed);
        assert_eq!(check_code("X", &["N", "B"]), CodeCheck::NotAllowed);
        assert_eq!(check_code("", &["N", "B"]), CodeCheck::UndefinedMandatory);
    }

    #[test]
    fn test_report_tallies_findings() {
        let mut report = ValidationReport::new("B1");
        report.add_finding(Finding {
            rule: VrRule::Vr08,
            severity: Severity::Error,
            outcome: Outcome::RowQuarantined,
            file: "ALTHVD".to_string(),
            line: 3,
            key: "7AC".to_string(),
            reason: "orphan".to_string(),
        });
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.warning_count(), 0);
        let tally = report.tallies.iter().find(|t| t.rule == "VR-08").unwrap();
        assert_eq!(tally.errors, 1);
    }

    #[test]
    fn test_status_is_recomputed_from_outcomes() {
        let mut report = ValidationReport::new("B1");
        report.files.push(FileOutcome {
            prefix: "ALTHVD".to_string(),
            source_file: "a.csv".to_string(),
            sha256: String::new(),
            filename_ok: true,
            header_ok: true,
            rows_total: 9,
            rows_accepted: 0,
            rows_quarantined: 9,
            repairs_applied: 0,
            rejected: false,
            reject_reason: String::new(),
        });
        report.recompute_status();
        assert_eq!(report.status, BatchStatus::PartialAccepted);
    }
}

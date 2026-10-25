#![forbid(unsafe_code)]

//! Wellfit MTRIX - Numbered, Source-Traceable Repair Rules.
//!
//! Every repair the engine performs is registered here with a stable identifier,
//! the source that authorises it, and the validation rule it belongs to. The
//! engine may only apply rules listed in [`REPAIR_RULES`]; anything else is a bug.
//!
//! Repairs are *targeted*: a rule applies to the column it owns, never to every
//! column of the row. Applying a barcode rule to an identifier column would
//! silently corrupt master data, so the targeted approach is a correctness
//! requirement, not a style choice.

use crate::spec::{FileSpec, KeyRole};
use serde::{Deserialize, Serialize};
use wellfit_core::mdm::MdmHub;

/// One authorised repair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepairRule {
    /// Stable identifier, e.g. `ANOM-06`.
    pub id: &'static str,
    /// Short description.
    pub title: &'static str,
    /// Authorising source, e.g. `L1 p.19` or `L2 MTRIX-019`.
    pub source: &'static str,
    /// Related validation rule, e.g. `VR-02`.
    pub rule: &'static str,
}

/// The complete catalogue of authorised repairs.
pub static REPAIR_RULES: &[RepairRule] = &[
    RepairRule {
        id: "ANOM-01",
        title: "Trailing empty field / delimiter residue",
        source: "L1 p.3-5 semicolon delimiter",
        rule: "VR-01",
    },
    RepairRule {
        id: "ANOM-02",
        title: "Address column spelling (ADRESS -> ADDRESS)",
        source: "L1 p.19",
        rule: "VR-02",
    },
    RepairRule {
        id: "ANOM-03",
        title: "Technical column alias (ZIP CODE / COUNTRY ISO / CUSTOMER ID)",
        source: "L2 naming standardisation",
        rule: "VR-02",
    },
    RepairRule {
        id: "ANOM-04",
        title: "Store type alias suspicion reported, the source value is kept",
        source: "L2 MTRIX-029 (show the source code)",
        rule: "VR-15",
    },
    RepairRule {
        id: "ANOM-05",
        title: "Vendor country prefix separated from vendor code",
        source: "L2 MTRIX-019",
        rule: "VR-01",
    },
    RepairRule {
        id: "ANOM-06",
        title: "Distributor dot mask removed",
        source: "L1 p.19 unmasked distributor RUC",
        rule: "VR-01",
    },
    RepairRule {
        id: "ANOM-07",
        title: "Barcode trailing noise removed",
        source: "observed interface bytes",
        rule: "VR-07",
    },
    RepairRule {
        id: "ANOM-08",
        title: "Negative stock clamped to zero (warning issued)",
        source: "L1 p.17 / VR-06",
        rule: "VR-06",
    },
    RepairRule {
        id: "ANOM-09",
        title: "Cell whitespace trimmed",
        source: "L1 p.3-5 no padding, no zero fill",
        rule: "VR-02",
    },
    RepairRule {
        id: "ANOM-10",
        title: "Duplicate sales transaction aggregated",
        source: "L2 VR-05",
        rule: "VR-05",
    },
    RepairRule {
        id: "ANOM-11",
        title: "Missing trailing columns padded with empty values",
        source: "L2 VR-02",
        rule: "VR-02",
    },
    RepairRule {
        id: "ANOM-12",
        title: "Canonical header emitted for downstream BI",
        source: "L2 BI model",
        rule: "VR-02",
    },
];

/// Identifier of the rule that removes trailing empty fields.
pub const ANOM_01: &str = "ANOM-01";
/// Identifier of the rule that normalises the address column name.
pub const ANOM_02: &str = "ANOM-02";
/// Identifier of the rule that applies technical column aliases.
pub const ANOM_03: &str = "ANOM-03";
/// Identifier of the rule that normalises store types.
pub const ANOM_04: &str = "ANOM-04";
/// Identifier of the rule that separates the vendor country prefix.
pub const ANOM_05: &str = "ANOM-05";
/// Identifier of the rule that removes the distributor dot mask.
pub const ANOM_06: &str = "ANOM-06";
/// Identifier of the rule that cleans barcode noise.
pub const ANOM_07: &str = "ANOM-07";
/// Identifier of the rule that clamps negative stock.
pub const ANOM_08: &str = "ANOM-08";
/// Identifier of the rule that trims cell whitespace.
pub const ANOM_09: &str = "ANOM-09";
/// Identifier of the rule that aggregates duplicate transactions.
pub const ANOM_10: &str = "ANOM-10";
/// Identifier of the rule that pads missing trailing columns.
pub const ANOM_11: &str = "ANOM-11";
/// Identifier of the rule that emits the canonical header.
pub const ANOM_12: &str = "ANOM-12";

/// A single applied repair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppliedRepair {
    /// Rule identifier from [`REPAIR_RULES`].
    pub rule_id: String,
    /// File prefix the repair was applied to.
    pub file: String,
    /// 1-based line number (0 for the header).
    pub line: usize,
    /// Column name the repair touched.
    pub column: String,
    /// Value before the repair.
    pub before: String,
    /// Value after the repair.
    pub after: String,
}

/// Collects the repairs applied while processing a drop.
#[derive(Debug, Default, Clone)]
pub struct RepairLog {
    applied: Vec<AppliedRepair>,
}

impl RepairLog {
    /// Every applied repair.
    pub fn applied(&self) -> &[AppliedRepair] {
        &self.applied
    }

    /// Number of applied repairs.
    pub fn count(&self) -> usize {
        self.applied.len()
    }

    /// Count of applied repairs for one rule.
    pub fn count_of(&self, rule_id: &str) -> usize {
        self.applied.iter().filter(|r| r.rule_id == rule_id).count()
    }

    fn record(
        &mut self,
        rule_id: &str,
        file: &str,
        line: usize,
        column: &str,
        before: &str,
        after: &str,
    ) {
        self.applied.push(AppliedRepair {
            rule_id: rule_id.to_string(),
            file: file.to_string(),
            line,
            column: column.to_string(),
            before: before.to_string(),
            after: after.to_string(),
        });
    }
}

/// Remove trailing empty fields, reporting whether anything was removed.
pub fn strip_trailing_empty(values: &mut Vec<String>) -> bool {
    let mut removed = false;
    while values.len() > 1 && values.last().map(|v| v.trim().is_empty()).unwrap_or(false) {
        values.pop();
        removed = true;
    }
    removed
}

/// Split a vendor identifier into its ISO country prefix and the vendor code.
///
/// The original manual specifies a two letter ISO country code followed by the
/// manufacturer code (14 positions in total). The delivered inventory file
/// carries a three letter prefix (`USA6063343900011222`), so both lengths are
/// accepted; the prefix is recorded, never silently discarded.
pub fn split_vendor_country_prefix(value: &str) -> (String, String) {
    let clean = value.trim();
    let bytes = clean.as_bytes();
    let prefix_len = if bytes.len() > 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1].is_ascii_alphabetic()
        && bytes[2].is_ascii_alphabetic()
        && bytes[3].is_ascii_digit()
    {
        3
    } else if bytes.len() > 2
        && bytes[0].is_ascii_alphabetic()
        && bytes[1].is_ascii_alphabetic()
        && bytes[2].is_ascii_digit()
    {
        2
    } else {
        0
    };
    if prefix_len == 0 {
        return (clean.to_string(), String::new());
    }
    let code = clean[prefix_len..].to_string();
    if code.len() < 12 || !code.chars().all(|c| c.is_ascii_digit()) {
        return (clean.to_string(), String::new());
    }
    (code, clean[..prefix_len].to_uppercase())
}

/// Build the canonical header for a file, recording the header-level repairs.
///
/// The canonical header is what downstream BI consumes; the deviations between
/// the delivered header and the original manual are reported separately so that
/// nothing is silently rewritten.
pub fn repair_header(spec: &FileSpec, raw_header: &str) -> (Vec<String>, RepairLog) {
    let mut log = RepairLog::default();
    let mut values: Vec<String> = raw_header
        .split(';')
        .map(|s| s.trim().to_string())
        .collect();

    if strip_trailing_empty(&mut values) {
        log.record(
            ANOM_01,
            spec.prefix,
            0,
            "<delimiter>",
            "trailing empty field",
            "removed",
        );
    }

    let mut header = Vec::with_capacity(spec.columns.len());
    for (idx, col) in spec.columns.iter().enumerate() {
        let delivered = values.get(idx).cloned().unwrap_or_default();
        let canonical = col.canonical_name.to_string();
        if !delivered.is_empty() && delivered != canonical {
            log.record(ANOM_03, spec.prefix, 0, &delivered, &delivered, &canonical);
        }
        header.push(canonical);
    }

    // ANOM-12: the emitted header is always the canonical one.
    log.record(
        ANOM_12,
        spec.prefix,
        0,
        "<header>",
        &values.join(";"),
        &header.join(";"),
    );
    (header, log)
}

/// Apply all authorised repairs to one data row.
///
/// Returns the repaired values, aligned to the file's declared column count.
pub fn repair_row(
    spec: &FileSpec,
    file: &str,
    line_no: usize,
    raw: &str,
    log: &mut RepairLog,
) -> Vec<String> {
    let mut values: Vec<String> = raw.split(';').map(|s| s.trim().to_string()).collect();

    if strip_trailing_empty(&mut values) {
        log.record(
            ANOM_01,
            file,
            line_no,
            "<delimiter>",
            "trailing empty field",
            "removed",
        );
    }

    while values.len() < spec.column_count() {
        values.push(String::new());
        log.record(
            ANOM_11,
            file,
            line_no,
            "<padding>",
            "missing trailing column",
            "empty",
        );
    }
    if values.len() > spec.column_count() {
        // The caller rejects the file (VR-02) before reaching this point; truncate
        // defensively so downstream code always sees a well-shaped row.
        values.truncate(spec.column_count());
    }

    for (idx, col) in spec.columns.iter().enumerate() {
        let current = values[idx].clone();
        let trimmed = current.trim().to_string();
        if trimmed != current {
            log.record(ANOM_09, file, line_no, col.source_name, &current, &trimmed);
            values[idx] = trimmed;
        }

        match col.key_role {
            KeyRole::Vendor => {
                let (code, prefix) = split_vendor_country_prefix(&values[idx]);
                if !prefix.is_empty() {
                    log.record(ANOM_05, file, line_no, col.source_name, &values[idx], &code);
                    values[idx] = code;
                }
            }
            KeyRole::Dist => {
                let unmasked = MdmHub::normalize_dist_id(&values[idx]);
                if unmasked != values[idx] {
                    log.record(
                        ANOM_06,
                        file,
                        line_no,
                        col.source_name,
                        &values[idx],
                        &unmasked,
                    );
                    values[idx] = unmasked;
                }
            }
            KeyRole::ProductBarcode => {
                let cleaned = values[idx].trim().trim_end_matches(';').trim().to_string();
                if cleaned != values[idx] {
                    log.record(
                        ANOM_07,
                        file,
                        line_no,
                        col.source_name,
                        &values[idx],
                        &cleaned,
                    );
                    values[idx] = cleaned;
                }
            }
            _ => {}
        }

        if col.source_name == "QUANTITY_STOCK" {
            if let Ok(value) = wellfit_core::scaled::ScaledDecimal::parse(&values[idx]) {
                if value.is_negative() {
                    log.record(
                        ANOM_08,
                        file,
                        line_no,
                        col.source_name,
                        &values[idx],
                        "0.0000",
                    );
                    values[idx] = "0".to_string();
                }
            }
        }
    }

    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{spec_by_role, FileRole};

    #[test]
    fn test_rule_catalogue_is_numbered_and_unique() {
        let mut ids: Vec<&str> = REPAIR_RULES.iter().map(|r| r.id).collect();
        ids.sort_unstable();
        let unique = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), unique, "duplicate repair rule id");
        for rule in REPAIR_RULES {
            assert!(
                rule.source.starts_with("L1")
                    || rule.source.starts_with("L2")
                    || rule.source.starts_with("observed")
                    || rule.source.starts_with("L1 ")
            );
            assert!(rule.rule.starts_with("VR-"));
        }
    }

    #[test]
    fn test_vendor_prefix_is_split_only_when_present() {
        assert_eq!(
            split_vendor_country_prefix("USA6063343900011222"),
            ("6063343900011222".to_string(), "USA".to_string())
        );
        assert_eq!(
            split_vendor_country_prefix("6063343900011222"),
            ("6063343900011222".to_string(), String::new())
        );
    }

    #[test]
    fn test_vendor_prefix_accepts_two_and_three_letter_forms() {
        // The manual specifies a two letter ISO prefix ...
        assert_eq!(
            split_vendor_country_prefix("BR6063343900011222"),
            ("6063343900011222".to_string(), "BR".to_string())
        );
        // ... while the delivered inventory file uses three letters.
        assert_eq!(
            split_vendor_country_prefix("USA6063343900011222"),
            ("6063343900011222".to_string(), "USA".to_string())
        );
        // A short alphabetic value is not a country prefix.
        assert_eq!(
            split_vendor_country_prefix("AB12"),
            ("AB12".to_string(), String::new())
        );
    }

    #[test]
    fn test_inventory_vendor_prefix_repair_targets_vendor_column_only() {
        let spec = spec_by_role(FileRole::Inventory);
        let mut log = RepairLog::default();
        let row = repair_row(
            spec,
            "ALTHES",
            2,
            "20240221;USA6063343900011222;029864857023000;7890541735003;167;USA",
            &mut log,
        );
        assert_eq!(row[1], "6063343900011222");
        // The inventory date must not be touched.
        assert_eq!(row[0], "20240221");
        assert_eq!(log.count_of(ANOM_05), 1);
    }

    #[test]
    fn test_negative_stock_is_clamped_with_a_warning_rule() {
        let spec = spec_by_role(FileRole::Inventory);
        let mut log = RepairLog::default();
        let row = repair_row(
            spec,
            "ALTHES",
            2,
            "20240221;6063343900011222;029864857023000;7890541735003;-12;USA",
            &mut log,
        );
        assert_eq!(row[4], "0");
        assert_eq!(log.count_of(ANOM_08), 1);
    }

    #[test]
    fn test_barcode_repair_does_not_touch_identifier_columns() {
        let spec = spec_by_role(FileRole::Product);
        let mut log = RepairLog::default();
        let row = repair_row(
            spec,
            "ALTHPD",
            2,
            "20240224;02.986.485.7023.000;6063343900011222;Alpargatas SE;8;0;7898656600087 ;1;;RSF;A;USA",
            &mut log,
        );
        assert_eq!(row[6], "7898656600087");
        // DIST_ID dot mask removed, but the 16 digit vendor code is untouched.
        assert_eq!(row[1], "029864857023000");
        assert_eq!(row[2], "6063343900011222");
    }

    #[test]
    fn test_store_type_value_is_never_rewritten() {
        let spec = spec_by_role(FileRole::Customer);
        let mut log = RepairLog::default();
        let row = repair_row(
            spec,
            "ALTHCI",
            5,
            "6063343900011222;029864857023000;995D;Tiktok Shop;;;;Bogor;;;;995D;Online;Onlinne;0;USA",
            &mut log,
        );
        // The source value is kept: the code table is unknown (MTRIX-029) and the
        // mandatory-ness of the column is undefined (MTRIX-003). A suspected
        // alias is reported as a finding, not written into the data.
        assert_eq!(row[13], "Onlinne");
        assert_eq!(log.count_of(ANOM_04), 0);
    }

    #[test]
    fn test_empty_store_type_stays_empty() {
        let spec = spec_by_role(FileRole::Customer);
        let mut log = RepairLog::default();
        let row = repair_row(
            spec,
            "ALTHCI",
            5,
            "6063343900011222;029864857023000;995D;Tiktok Shop;;;;Bogor;;;;995D;Online;;0;USA",
            &mut log,
        );
        assert_eq!(row[13], "");
    }

    #[test]
    fn test_header_emits_canonical_names() {
        let spec = spec_by_role(FileRole::Customer);
        let (header, log) = repair_header(
            spec,
            "VENDOR_ID;DIST_ID;CUSTOMER_ID;CUSTOMER_NAME;ADRESS;DISTRICT;ZIP_CODE;CITY;STATE;SALES_CONTACT;PHONE_NUMBERS;CUSTOMER_ID;ROUTE;STORE_TYPE;PROPORTION;COUNTRY_ISO;",
        );
        assert_eq!(header[4], "ADDRESS");
        assert_eq!(header[6], "ZIP_CODE");
        assert_eq!(header[11], "CUSTOMER_ID_2");
        assert_eq!(header.len(), 16);
        assert!(log.count_of(ANOM_01) >= 1);
    }
}

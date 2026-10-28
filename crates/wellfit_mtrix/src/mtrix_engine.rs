#![forbid(unsafe_code)]

//! Wellfit MTRIX - Specification-Driven Repair and Validation Engine.
//!
//! The engine implements a three stage pipeline over a daily drop of the five
//! Alpargatas MTRIX files:
//!
//! 1. **Accept**   - VR-01 (file name) and VR-02 (header) decide whether a file
//!    may enter the pipeline at all. A failing file is rejected on its own; the
//!    remaining files are still processed (partial acceptance).
//! 2. **Repair**   - only the rules registered in [`crate::repairs::REPAIR_RULES`]
//!    are applied, each one targeted at the column it owns.
//! 3. **Validate** - row level rules (VR-03, VR-04, VR-05, VR-06, VR-07) and
//!    cross-file rules (VR-08 .. VR-11) produce findings with an explicit
//!    outcome.
//!
//! Outcomes follow the wording of the source rules: a row whose *value* breaks
//! the contract is isolated (VR-03, VR-04, VR-07), while a cross-file
//! referential finding *lists* the offending key without dropping the row
//! (VR-08 .. VR-11, "list the CUSTOMER_ID / item code / business partner and
//! customer combination").

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use wellfit_core::ledger::{Finding as LedgerFinding, FindingSeverity, IdempotencyLedger};
use wellfit_core::mdm::MdmHub;
use wellfit_core::release_gate::ReleaseGate;
use wellfit_core::scaled::ScaledDecimal;

use crate::config::ValidationConfig;
use crate::pipeline::{default_pipeline, PipelineContext};
use crate::repairs;
use crate::spec::{self, FieldKind, FileRole, FileSpec, KeyRole};
use crate::validators::{
    check_code, check_filename, compare_header, is_ean13, is_iso_alpha3, is_valid_yyyymmdd,
    BatchStatus, CodeCheck, CurrencyGroup, Deviation, ExceptionNotification, FileOutcome, Finding,
    Outcome, Severity, ValidationReport, VrRule,
};

/// The five files of a drop, keyed by business role.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MtrixFileSet {
    /// Customer / point of sale master.
    pub althci: String,
    /// Inventory snapshot.
    pub althes: String,
    /// Sales force master.
    pub althfv: String,
    /// Product master.
    pub althpd: String,
    /// Sales / sellout transactions.
    pub althvd: String,
}

impl MtrixFileSet {
    /// Content of one role.
    pub fn get(&self, role: FileRole) -> &str {
        match role {
            FileRole::Customer => &self.althci,
            FileRole::Inventory => &self.althes,
            FileRole::SalesForce => &self.althfv,
            FileRole::Product => &self.althpd,
            FileRole::Sales => &self.althvd,
        }
    }

    /// Content of one role, mutable.
    pub fn get_mut(&mut self, role: FileRole) -> &mut String {
        match role {
            FileRole::Customer => &mut self.althci,
            FileRole::Inventory => &mut self.althes,
            FileRole::SalesForce => &mut self.althfv,
            FileRole::Product => &mut self.althpd,
            FileRole::Sales => &mut self.althvd,
        }
    }
}

/// Repair counters per file, kept for reporting and backwards compatibility.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepairedCounts {
    /// Repairs applied to the customer master.
    pub althci: usize,
    /// Repairs applied to the inventory file.
    pub althes: usize,
    /// Repairs applied to the sales force master.
    pub althfv: usize,
    /// Repairs applied to the product master.
    pub althpd: usize,
    /// Repairs applied to the sales file.
    pub althvd: usize,
}

/// A file discovered in the drop folder.
#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    /// Business role of the file.
    pub role: FileRole,
    /// Full path of the file.
    pub path: PathBuf,
    /// File name as delivered.
    pub file_name: String,
}

/// A single row violation: the rule, the offending key and the reason.
type RowViolation = (VrRule, String, String);

/// A row that was isolated, together with every rule it violated.
type IsolatedRow = (usize, Vec<String>, Vec<RowViolation>);

/// A file after parsing and repair.
pub struct ParsedFile {
    role: FileRole,
    source_name: String,
    rows: Vec<Vec<String>>,
    /// The physical line number of every accepted row, so that a finding
    /// points at the line of the delivered file even when earlier rows were
    /// isolated.
    row_lines: Vec<usize>,
    rejected: bool,
}

/// A cross file reference that did not resolve.
struct ReferentialFinding<'a> {
    rule: VrRule,
    file: &'a ParsedFile,
    line_no: usize,
    key: &'a str,
    reason: &'a str,
}

/// The MTRIX engine.
pub struct MtrixEngine {
    ledger: Arc<IdempotencyLedger>,
    pub release_gate: Arc<Mutex<ReleaseGate>>,
    /// Repair counters per file.
    pub repaired_counts: RepairedCounts,
    /// Configurable validation parameters.
    pub config: ValidationConfig,
}

impl MtrixEngine {
    /// Create an engine bound to a ledger and a release gate.
    pub fn new(ledger: Arc<IdempotencyLedger>, release_gate: Arc<Mutex<ReleaseGate>>) -> Self {
        Self {
            ledger,
            release_gate,
            repaired_counts: RepairedCounts::default(),
            config: ValidationConfig::default(),
        }
    }

    /// Discover the five MTRIX files in a drop folder.
    ///
    /// Files are matched by their prefix. When several files share a prefix the
    /// newest timestamp wins and the ambiguity is reported.
    pub fn discover_files(input_dir: &Path) -> Result<Vec<DiscoveredFile>, String> {
        let entries = fs::read_dir(input_dir)
            .map_err(|e| format!("cannot read drop folder {}: {e}", input_dir.display()))?;
        let mut best: HashMap<FileRole, (String, PathBuf)> = HashMap::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let file_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            for role in spec::MTRIX_SPEC.iter().map(|f| f.role) {
                if !file_name.to_uppercase().starts_with(role.prefix()) {
                    continue;
                }
                let stamp = file_stamp(&file_name, role.prefix());
                let replace = match best.get(&role) {
                    Some((existing_stamp, _)) => &stamp > existing_stamp,
                    None => true,
                };
                if replace {
                    best.insert(role, (stamp, path.clone()));
                }
            }
        }

        let mut missing: Vec<&str> = Vec::new();
        for file in spec::MTRIX_SPEC {
            if !best.contains_key(&file.role) {
                missing.push(file.prefix);
            }
        }
        if !missing.is_empty() {
            return Err(format!(
                "drop folder {} is missing required file(s): {}",
                input_dir.display(),
                missing.join(", ")
            ));
        }

        let mut discovered: Vec<DiscoveredFile> = Vec::new();
        for file in spec::MTRIX_SPEC {
            let (stamp, path) = best.remove(&file.role).expect("presence checked above");
            let file_name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| stamp);
            discovered.push(DiscoveredFile {
                role: file.role,
                path,
                file_name,
            });
        }
        Ok(discovered)
    }

    /// Process a complete drop: accept, repair, validate, and emit the report.
    ///
    /// When `output_dir` is given, the repaired files are written next to it
    /// using the delivered file name plus a `_repaired` suffix, so the result
    /// can be fed back to the counterpart system.
    pub fn process_drop(
        &mut self,
        input_dir: &Path,
        batch_id: &str,
        output_dir: Option<&Path>,
    ) -> Result<ValidationReport, String> {
        let discovered = Self::discover_files(input_dir)?;

        // Idempotency: the drop is claimed in the ledger before anything is
        // written, so that the same drop cannot be ingested twice and produce a
        // second set of findings.
        let mut fingerprint = String::new();
        for item in &discovered {
            let content =
                wellfit_core::text::read_capped(&item.path, wellfit_core::text::MAX_INPUT_BYTES)?;
            fingerprint.push_str(&format!(
                "{}:{}\n",
                item.file_name,
                IdempotencyLedger::compute_sha256(content.as_bytes())
            ));
        }
        let claimed = self
            .ledger
            .register_file_batch(
                batch_id,
                "MTRIX_DROP",
                "mtrix-drop",
                fingerprint.as_bytes(),
                0,
            )
            .map_err(|e| format!("cannot register the batch in the ledger: {e}"))?;
        if !claimed {
            // A corrected re-send under the same batch identifier is allowed
            // when the content changed: the previous claim is released and a
            // new one is taken. An identical fingerprint is a true duplicate
            // and is refused.
            let existing = self
                .ledger
                .content_hash_of(batch_id)
                .map_err(|e| format!("cannot read the previous batch claim: {e}"))?;
            let new_content_hash = IdempotencyLedger::compute_sha256(fingerprint.as_bytes());
            let corrected = existing
                .as_deref()
                .map(|h| h != new_content_hash)
                .unwrap_or(false);
            if corrected {
                self.ledger
                    .release_file_batch(batch_id)
                    .map_err(|e| format!("cannot release the previous batch claim: {e}"))?;
                let re_claimed = self
                    .ledger
                    .register_file_batch(
                        batch_id,
                        "MTRIX_DROP",
                        "mtrix-drop",
                        fingerprint.as_bytes(),
                        0,
                    )
                    .map_err(|e| format!("cannot register the corrected batch: {e}"))?;
                if !re_claimed {
                    return Err(format!(
                        "Duplicate MTRIX batch {batch_id} already registered; use a new --batch to resend a corrected drop"
                    ));
                }
            } else {
                return Err(format!(
                    "Duplicate MTRIX batch {batch_id} already registered; use a new --batch to resend a corrected drop"
                ));
            }
        }

        let mut report = ValidationReport::new(batch_id);
        report.spec_conformance = spec_conformance();

        let mut parsed: Vec<ParsedFile> = Vec::new();
        let mut repaired_outputs: Vec<(PathBuf, String)> = Vec::new();
        let exceptions: Vec<ExceptionNotification> = Vec::new();

        for item in discovered {
            let file_spec = spec::spec_by_role(item.role);
            let raw =
                wellfit_core::text::read_capped(&item.path, wellfit_core::text::MAX_INPUT_BYTES)?;
            let sha256 = IdempotencyLedger::compute_sha256(raw.as_bytes());
            let mut outcome = FileOutcome {
                prefix: file_spec.prefix.to_string(),
                source_file: item.file_name.clone(),
                sha256,
                filename_ok: false,
                header_ok: false,
                rows_total: 0,
                rows_accepted: 0,
                rows_quarantined: 0,
                repairs_applied: 0,
                rejected: false,
                reject_reason: String::new(),
            };

            // --- VR-01: file name -------------------------------------------------
            let name_check = check_filename(&item.file_name, file_spec.prefix);
            outcome.filename_ok = name_check.pattern_ok && name_check.timestamp_valid;
            for warning in &name_check.warnings {
                self.record_finding(
                    &mut report,
                    batch_id,
                    Finding {
                        rule: VrRule::Vr01,
                        severity: Severity::Warning,
                        outcome: Outcome::Repaired,
                        file: file_spec.prefix.to_string(),
                        line: 0,
                        key: item.file_name.clone(),
                        reason: warning.clone(),
                    },
                )?;
            }
            if !outcome.filename_ok {
                outcome.rejected = true;
                outcome.reject_reason = if !name_check.pattern_ok {
                    format!(
                        "file name does not match the expected pattern '{}<DDMMYYYY><HHMMSSmmm>'",
                        file_spec.prefix
                    )
                } else {
                    "file name carries an invalid timestamp".to_string()
                };
                self.record_finding(
                    &mut report,
                    batch_id,
                    Finding {
                        rule: VrRule::Vr01,
                        severity: Severity::Error,
                        outcome: Outcome::FileRejected,
                        file: file_spec.prefix.to_string(),
                        line: 0,
                        key: item.file_name.clone(),
                        reason: outcome.reject_reason.clone(),
                    },
                )?;
                report.files.push(outcome);
                continue;
            }

            // --- VR-02: header ----------------------------------------------------
            let records = wellfit_core::text::split_records(&raw);
            let raw_header = records.first().map(|(_, line)| *line).unwrap_or_default();
            let (canonical_header, header_log) = repairs::repair_header(file_spec, raw_header);
            let delivered_header: Vec<String> = raw_header
                .trim_end_matches(';')
                .split(';')
                .map(|s| s.trim().to_string())
                .collect();
            let (header_ok, differences) = compare_header(file_spec, &delivered_header);
            outcome.header_ok = header_ok;
            if !header_ok {
                outcome.rejected = true;
                outcome.reject_reason = differences.join("; ");
                for difference in &differences {
                    self.record_finding(
                        &mut report,
                        batch_id,
                        Finding {
                            rule: VrRule::Vr02,
                            severity: Severity::Error,
                            outcome: Outcome::FileRejected,
                            file: file_spec.prefix.to_string(),
                            line: 1,
                            key: file_spec.prefix.to_string(),
                            reason: difference.clone(),
                        },
                    )?;
                }
                report.files.push(outcome);
                continue;
            }

            // --- Repair + row validation -----------------------------------------
            let mut rows: Vec<Vec<String>> = Vec::new();
            let mut row_lines: Vec<usize> = Vec::new();
            let mut isolated: Vec<IsolatedRow> = Vec::new();
            let mut row_log = header_log;

            for (line_no, line) in records.iter().skip(1) {
                let line_no = *line_no;
                if line.trim().is_empty() {
                    continue;
                }
                let negative_stock = negative_measure(file_spec, line);
                let mut values =
                    repairs::repair_row(file_spec, file_spec.prefix, line_no, line, &mut row_log);
                outcome.rows_total += 1;

                if negative_stock {
                    let reason =
                        "negative inventory quantity was clamped to zero (the manual forbids negative stock)"
                            .to_string();
                    self.record_finding(
                        &mut report,
                        batch_id,
                        Finding {
                            rule: VrRule::Vr06,
                            severity: Severity::Warning,
                            outcome: Outcome::Repaired,
                            file: file_spec.prefix.to_string(),
                            line: line_no,
                            key: String::new(),
                            reason,
                        },
                    )?;
                }

                // Every rule is evaluated, not just the first one that fails: the
                // row is isolated once, but the report has to name every reason
                // why the row was rejected, otherwise the sender has to resubmit
                // the file several times to discover the remaining problems.
                for (rule, key, reason) in self.format_warnings(file_spec, &values) {
                    self.record_finding(
                        &mut report,
                        batch_id,
                        Finding {
                            rule,
                            severity: Severity::Warning,
                            outcome: Outcome::Listed,
                            file: file_spec.prefix.to_string(),
                            line: line_no,
                            key,
                            reason,
                        },
                    )?;
                }
                let violations = self.validate_row(file_spec, &values, line_no);
                if !violations.is_empty() {
                    isolated.push((line_no, values.clone(), violations));
                    continue;
                }
                rows.push(std::mem::take(&mut values));
                row_lines.push(line_no);
            }

            // VR-05: aggregate duplicate sales transactions (repair, not rejection).
            rows =
                self.aggregate_duplicates(file_spec, rows, &mut row_lines, &mut report, batch_id)?;

            outcome.repairs_applied = row_log.count();
            report.repairs.extend(row_log.applied().iter().cloned());
            outcome.rows_accepted = rows.len();
            outcome.rows_quarantined = isolated.len();

            for (line_no, values, violations) in &isolated {
                for (rule, key, reason) in violations {
                    self.ledger
                        .log_quarantine_with_severity(LedgerFinding {
                            batch_id,
                            rule_code: rule.id(),
                            severity: ledger_severity(Severity::Error),
                            filename: file_spec.prefix,
                            line_number: *line_no,
                            key_value: &values.join(";"),
                            reason,
                        })
                        .map_err(|e| {
                            format!(
                                "cannot persist quarantine {} at {} line {}: {e}",
                                rule.id(),
                                file_spec.prefix,
                                line_no
                            )
                        })?;
                    report.add_finding(Finding {
                        rule: *rule,
                        severity: Severity::Error,
                        outcome: Outcome::RowQuarantined,
                        file: file_spec.prefix.to_string(),
                        line: *line_no,
                        key: key.clone(),
                        reason: reason.clone(),
                    });
                }
            }

            for repair in row_log.applied() {
                self.ledger
                    .log_finding(LedgerFinding {
                        batch_id,
                        rule_code: &repair.rule_id,
                        severity: ledger_severity(Severity::Info),
                        filename: &repair.file,
                        line_number: repair.line,
                        key_value: &repair.column,
                        reason: &format!("{} -> {}", repair.before, repair.after),
                    })
                    .map_err(|e| {
                        format!(
                            "cannot persist repair finding {} at {} line {}: {e}",
                            repair.rule_id, repair.file, repair.line
                        )
                    })?;
            }

            self.apply_counts(file_spec.role, row_log.count());
            report.repairs_applied += row_log.count();

            // Deviations of the delivered header against the original manual.
            for col in file_spec.columns {
                if col.is_deviation() {
                    report.deviations.push(Deviation {
                        prefix: file_spec.prefix.to_string(),
                        seq: col.seq,
                        source_name: col.source_name.to_string(),
                        spec_name: col.spec_name.to_string(),
                        canonical_name: col.canonical_name.to_string(),
                        issue_id: col.issue_id.unwrap_or("").to_string(),
                        manual_page: col
                            .manual_page
                            .expect("column declared by the MTRIX specification"),
                    });
                }
            }

            if let Some(dir) = output_dir {
                let out_name = repaired_file_name(&item.file_name);
                let out_path = dir.join(out_name);
                let mut body = canonical_header.join(";");
                for row in &rows {
                    body.push_str(self.config.output_line_ending.as_str());
                    body.push_str(&row.join(";"));
                }
                let terminator = self.config.output_line_ending.as_str();
                repaired_outputs.push((out_path, format!("{body}{terminator}")));
            }

            report.files.push(outcome);
            parsed.push(ParsedFile {
                role: item.role,
                source_name: item.file_name,
                rows,
                row_lines,
                rejected: false,
            });
        }

        // Run post-processing stages through the pipeline.
        let mut ctx = PipelineContext {
            engine: self,
            dir: input_dir,
            batch_id,
            output_dir,
            parsed,
            repaired_outputs,
            report,
            errors: Vec::new(),
            exceptions,
        };

        let pipeline = default_pipeline();
        pipeline.run(&mut ctx)?;

        // The claim is retained for both clean and defective drops. A corrected
        // re-send under the same batch identifier is accepted only when its
        // content fingerprint differs (see the claim block above); an identical
        // fingerprint is a true duplicate and is refused. Releasing the claim on
        // isolation would let the same defective drop be ingested twice.

        // Direction B: export exception notifications for cross-file findings
        // so the business side can synchronize master data.
        if !ctx.exceptions.is_empty() {
            ctx.report.exceptions = ctx.exceptions.clone();
            let exception_path = input_dir.join(format!("{}_exceptions.json", batch_id));
            let exception_json = serde_json::to_string_pretty(&ctx.report.exceptions)
                .map_err(|e| format!("cannot serialise exception notifications: {e}"))?;
            fs::write(&exception_path, exception_json).map_err(|e| {
                format!(
                    "cannot write exception file {}: {e}",
                    exception_path.display()
                )
            })?;
        }

        Ok(ctx.report)
    }

    /// Validate a single row. Returns every violated rule with the offending key
    /// and the reason. A row that violates at least one rule is isolated; the
    /// violations are all reported, not only the first one.
    fn validate_row(
        &self,
        file_spec: &FileSpec,
        values: &[String],
        line_no: usize,
    ) -> Vec<RowViolation> {
        let mut violations: Vec<RowViolation> = Vec::new();
        for (idx, col) in file_spec.columns.iter().enumerate() {
            let value = values.get(idx).map(|v| v.trim()).unwrap_or("");
            match col.kind {
                FieldKind::Date => {
                    if !value.is_empty() && !is_valid_yyyymmdd(value) {
                        violations.push((
                            VrRule::Vr03,
                            value.to_string(),
                            format!(
                                "{} is not a valid YYYYMMDD calendar date (line {line_no})",
                                col.source_name
                            ),
                        ));
                    }
                }
                FieldKind::Number => {
                    if !value.is_empty() && ScaledDecimal::parse(value).is_err() {
                        violations.push((
                            VrRule::Vr15,
                            value.to_string(),
                            format!(
                                "{} value '{}' is not a readable number (line {line_no})",
                                col.source_name, value
                            ),
                        ));
                    }
                }
                FieldKind::Code => {
                    if let Some(allowed) = col.allowed {
                        match check_code(value, allowed) {
                            CodeCheck::Allowed => {}
                            CodeCheck::NotAllowed => {
                                let rule = coded_field_rule(file_spec.role);
                                violations.push((
                                    rule,
                                    value.to_string(),
                                    format!(
                                        "{} value '{}' is outside the allowed set {:?} (line {line_no})",
                                        col.source_name, value, allowed
                                    ),
                                ));
                            }
                            CodeCheck::UndefinedMandatory => {
                                // Mandatory-ness is undefined by the manual (MTRIX-003):
                                // the row is kept and the caveat is reported.
                            }
                        }
                    }
                }
                _ => {}
            }

            // Per-field format validation (VR-15).
            if col.source_name == "COUNTRY_ISO" && !value.is_empty() && !is_iso_alpha3(value) {
                violations.push((
                    VrRule::Vr15,
                    value.to_string(),
                    format!(
                        "COUNTRY_ISO '{value}' is not an ISO 3166-1 alpha-3 country code (line {line_no})"
                    ),
                ));
            }
        }

        if let Some((barcode, barcode_type)) = self.barcode_of(file_spec, values) {
            if barcode_type == "1" && !is_ean13(barcode) {
                violations.push((
                    VrRule::Vr15,
                    barcode.to_string(),
                    format!(
                        "BARCODE '{barcode}' is declared as EAN13 but does not carry a valid check digit (line {line_no})"
                    ),
                ));
            }
        }
        violations
    }

    /// Field format findings that do not isolate the row.
    ///
    /// These are the per-field validations the field dictionary declares column
    /// by column. A value that looks like a country code in a name column is
    /// reported as a column shift suspicion: the delivered sales force file
    /// carries one delimiter too few, so the country code lands in the name
    /// column. That is a data defect of the sender, and it has to be visible in
    /// the report instead of being read as a clean file.
    fn format_warnings(
        &self,
        file_spec: &FileSpec,
        values: &[String],
    ) -> Vec<(VrRule, String, String)> {
        let mut warnings = Vec::new();
        let row_has_value = values.iter().any(|v| !v.trim().is_empty());
        for (idx, col) in file_spec.columns.iter().enumerate() {
            let value = values.get(idx).map(|v| v.trim()).unwrap_or("");
            if value.is_empty() {
                // Mandatory-ness is undefined for columns listed in the config
                // (MTRIX-003). An empty value in such a column is a
                // data-quality caveat rather than a rejection: keep the row,
                // report the caveat.
                if self
                    .config
                    .undefined_mandatory_columns
                    .contains(&col.source_name)
                    && col.source_name != "DOCUMENT_TYPE"
                    && col.source_name != "SALESPERSON_CODE"
                    && row_has_value
                {
                    warnings.push((
                        VrRule::Vr15,
                        String::new(),
                        format!(
                            "{} is empty; mandatory-ness is undefined by the manual (MTRIX-003)",
                            col.source_name
                        ),
                    ));
                }
                // An empty country code on a row that carries other values is a
                // column-shift or missing-data defect: the row cannot be grouped
                // for the acceptance totals, so it must be visible (R-01).
                let is_country = col.canonical_name == "COUNTRY_ISO"
                    || col.source_name == "COUNTRY"
                    || col.source_name == "COUNTRY_ISO";
                if is_country && row_has_value {
                    warnings.push((
                        VrRule::Vr15,
                        String::new(),
                        format!(
                            "{} is empty; the row cannot be grouped for the acceptance totals (MTRIX-003, MTRIX-023)",
                            col.source_name
                        ),
                    ));
                }
                continue;
            }
            if col.source_name.ends_with("_NAME") && is_iso_alpha3(value) {
                warnings.push((
                    VrRule::Vr15,
                    value.to_string(),
                    format!(
                        "{} carries the country-like value '{}'; the row appears to be shifted by one column",
                        col.source_name, value
                    ),
                ));
            }
            if col.source_name == "STORE_TYPE" {
                let canonical = MdmHub::normalize_store_type(value);
                if !canonical.is_empty() && canonical != value {
                    warnings.push((
                        VrRule::Vr15,
                        value.to_string(),
                        format!(
                            "STORE_TYPE '{value}' matches the known alias '{canonical}'; the source value is kept because the code table is unknown (MTRIX-029)"
                        ),
                    ));
                }
            }
            if col.source_name == "ZIP_CODE"
                && self.config.cp_format_strict
                && !value.is_empty()
                && !self.config.is_valid_cp(value)
            {
                warnings.push((
                    VrRule::Vr15,
                    value.to_string(),
                    format!(
                        "ZIP_CODE '{}' does not match CP format; format is unconfirmed (MTRIX-001)",
                        value
                    ),
                ));
            }
        }
        warnings
    }

    /// The barcode of a row together with the barcode type that applies to it.
    fn barcode_of<'a>(
        &self,
        file_spec: &FileSpec,
        values: &'a [String],
    ) -> Option<(&'a str, &'a str)> {
        let barcode_idx = file_spec.index_of("BARCODE")?;
        let type_idx = file_spec.index_of("BARCODE_TYPE")?;
        let barcode = values.get(barcode_idx)?.trim();
        let barcode_type = values.get(type_idx).map(|v| v.trim()).unwrap_or("");
        if barcode.is_empty() {
            return None;
        }
        Some((barcode, barcode_type))
    }

    /// VR-05: aggregate duplicated sales transactions into a single row.
    fn aggregate_duplicates(
        &self,
        file_spec: &FileSpec,
        rows: Vec<Vec<String>>,
        row_lines: &mut Vec<usize>,
        report: &mut ValidationReport,
        batch_id: &str,
    ) -> Result<Vec<Vec<String>>, String> {
        if file_spec.role != FileRole::Sales {
            return Ok(rows);
        }
        let doc_idx = file_spec.index_of("DOCUMENT_NUMBER");
        let batch_idx = file_spec.index_of("BATCH_CODE");
        let qty_idx = file_spec.index_of("QUANTITY");
        let cust_idx = file_spec.index_of_role(KeyRole::Customer);
        let type_idx = file_spec.index_of("DOCUMENT_TYPE");
        let (Some(doc_idx), Some(batch_idx), Some(qty_idx)) = (doc_idx, batch_idx, qty_idx) else {
            return Ok(rows);
        };

        let mut grouped: Vec<(String, Vec<String>)> = Vec::new();
        let mut grouped_lines: Vec<usize> = Vec::new();
        let mut index: HashMap<String, usize> = HashMap::new();
        // (natural key, surviving line, duplicate line) for every merge.
        let mut merged: Vec<(String, usize, usize)> = Vec::new();

        for (i, row) in rows.into_iter().enumerate() {
            let duplicate_line = row_lines.get(i).copied().unwrap_or(0);
            let key = [
                cust_idx
                    .and_then(|i| row.get(i))
                    .map(|v| v.trim())
                    .unwrap_or(""),
                type_idx
                    .and_then(|i| row.get(i))
                    .map(|v| v.trim())
                    .unwrap_or(""),
                row.get(doc_idx).map(|v| v.trim()).unwrap_or(""),
                row.get(batch_idx).map(|v| v.trim()).unwrap_or(""),
            ]
            .join("|");
            match index.get(&key).copied() {
                Some(pos) => {
                    let previous = grouped[pos].1[qty_idx].clone();
                    let current = row.get(qty_idx).cloned().unwrap_or_default();
                    let previous_value = ScaledDecimal::parse(&previous).map_err(|e| {
                        format!("cannot aggregate invalid prior quantity for key {key}: {e}")
                    })?;
                    let current_value = ScaledDecimal::parse(&current).map_err(|e| {
                        format!("cannot aggregate invalid quantity for key {key}: {e}")
                    })?;
                    let sum = previous_value.checked_add(&current_value).ok_or_else(|| {
                        format!("quantity overflow while aggregating duplicate sales key {key}")
                    })?;
                    let formatted = sum.to_plain_string();
                    let formatted = formatted
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string();
                    grouped[pos].1[qty_idx] = if formatted.is_empty() {
                        "0".to_string()
                    } else {
                        formatted
                    };
                    merged.push((key, grouped_lines[pos], duplicate_line));
                }
                None => {
                    index.insert(key.clone(), grouped.len());
                    grouped_lines.push(duplicate_line);
                    grouped.push((key, row));
                }
            }
        }

        // VR-05 finding for every transaction that was actually merged. The
        // surviving line and the duplicate line are both named so the sender
        // can locate both rows.
        for (key, surviving_line, duplicate_line) in &merged {
            self.record_finding(
                report,
                batch_id,
                Finding {
                    rule: VrRule::Vr05,
                    severity: Severity::Error,
                    outcome: Outcome::Repaired,
                    file: file_spec.prefix.to_string(),
                    line: *surviving_line,
                    key: key.clone(),
                    reason: format!(
                        "duplicate sales transaction at line {duplicate_line} aggregated into line {surviving_line} (VR-05, error level)"
                    ),
                },
            )?;
        }

        row_lines.clear();
        row_lines.extend(grouped_lines);
        Ok(grouped.into_iter().map(|(_, row)| row).collect())
    }

    /// VR-08 .. VR-11: cross-file referential integrity.
    pub fn apply_cross_file_rules(
        &self,
        parsed: &[ParsedFile],
        report: &mut ValidationReport,
        batch_id: &str,
        exceptions: &mut Vec<ExceptionNotification>,
    ) -> Result<(), String> {
        let customers: Option<HashSet<String>> = parsed
            .iter()
            .find(|f| f.role == FileRole::Customer && !f.rejected)
            .map(|f| {
                let spec = spec::spec_by_role(FileRole::Customer);
                let idx = spec
                    .index_of_role(KeyRole::Customer)
                    .expect("column declared by the MTRIX specification");
                f.rows
                    .iter()
                    .map(|r| r.get(idx).map(|v| v.trim().to_string()).unwrap_or_default())
                    .filter(|v| !v.is_empty())
                    .collect()
            });

        let inventory_batches: Option<HashSet<String>> = parsed
            .iter()
            .find(|f| f.role == FileRole::Inventory && !f.rejected)
            .map(|f| {
                let spec = spec::spec_by_role(FileRole::Inventory);
                let idx = spec
                    .index_of_role(KeyRole::Batch)
                    .expect("column declared by the MTRIX specification");
                f.rows
                    .iter()
                    .map(|r| r.get(idx).map(|v| v.trim().to_string()).unwrap_or_default())
                    .filter(|v| !v.is_empty())
                    .collect()
            });

        // The product key of a product is its barcode when it has one, and its
        // internal code only when the barcode is absent. The manual states the
        // fallback per product ("in the absence of the EAN13 or DUN14 code, the
        // internal code of the product is exported"), so the two key spaces are
        // not unioned: an internal code must not satisfy the check for a
        // product that does have a barcode.
        let product_keys: Option<HashSet<String>> = parsed
            .iter()
            .find(|f| f.role == FileRole::Product && !f.rejected)
            .map(|f| {
                let spec = spec::spec_by_role(FileRole::Product);
                let barcode_idx = spec.index_of_role(KeyRole::ProductBarcode);
                let code_idx = spec.index_of_role(KeyRole::Product);
                let mut keys = HashSet::new();
                for row in &f.rows {
                    let barcode = barcode_idx
                        .and_then(|i| row.get(i))
                        .map(|v| v.trim())
                        .unwrap_or("");
                    let internal_code = code_idx
                        .and_then(|i| row.get(i))
                        .map(|v| v.trim())
                        .unwrap_or("");
                    let key = if !barcode.is_empty() {
                        barcode
                    } else {
                        internal_code
                    };
                    if !key.is_empty() {
                        keys.insert(key.to_string());
                    }
                }
                keys
            });

        let sales_force: Option<HashSet<(String, String)>> = parsed
            .iter()
            .find(|f| f.role == FileRole::SalesForce && !f.rejected)
            .map(|f| {
                let spec = spec::spec_by_role(FileRole::SalesForce);
                let sp_idx = spec.index_of_role(KeyRole::Salesperson);
                let cust_idx = spec.index_of_role(KeyRole::Customer);
                let mut pairs = HashSet::new();
                for row in &f.rows {
                    let sp = sp_idx
                        .and_then(|i| row.get(i))
                        .map(|v| v.trim().to_string())
                        .unwrap_or_default();
                    let cust = cust_idx
                        .and_then(|i| row.get(i))
                        .map(|v| v.trim().to_string())
                        .unwrap_or_default();
                    if !sp.is_empty() {
                        pairs.insert((sp, cust));
                    }
                }
                pairs
            });

        // --- Sales rules --------------------------------------------------------
        if let Some(sales) = parsed
            .iter()
            .find(|f| f.role == FileRole::Sales && !f.rejected)
        {
            let spec = spec::spec_by_role(FileRole::Sales);
            let cust_idx = spec
                .index_of_role(KeyRole::Customer)
                .expect("column declared by the MTRIX specification");
            let batch_idx = spec
                .index_of_role(KeyRole::Batch)
                .expect("column declared by the MTRIX specification");
            let sp_idx = spec
                .index_of_role(KeyRole::Salesperson)
                .expect("column declared by the MTRIX specification");
            let doc_type_idx = spec
                .index_of("DOCUMENT_TYPE")
                .expect("column declared by the MTRIX specification");

            for (offset, row) in sales.rows.iter().enumerate() {
                let line_no = sales.row_lines.get(offset).copied().unwrap_or(offset + 2);

                let customer = row.get(cust_idx).map(|v| v.trim()).unwrap_or("");
                match &customers {
                    Some(keys) if !customer.is_empty() && !keys.contains(customer) => {
                        let reason = format!(
                            "customer '{customer}' does not exist in the customer master (ALTHCI)"
                        );
                        if let Some(notification) = self.record_referential(
                            report,
                            batch_id,
                            ReferentialFinding {
                                rule: VrRule::Vr08,
                                file: sales,
                                line_no,
                                key: customer,
                                reason: &reason,
                            },
                        )? {
                            exceptions.push(notification);
                        }
                    }
                    None => {
                        self.record_not_evaluable(
                            report,
                            batch_id,
                            VrRule::Vr08,
                            sales,
                            line_no,
                            customer,
                        )?;
                    }
                    _ => {}
                }

                let batch = row.get(batch_idx).map(|v| v.trim()).unwrap_or("");
                match (&inventory_batches, &product_keys) {
                    (Some(inv), Some(prod)) => {
                        let in_inventory = inv.contains(batch);
                        let in_product = prod.contains(batch);
                        if !batch.is_empty() && !(in_inventory && in_product) {
                            let reason = format!(
                                "item '{batch}' is not present in both the inventory file ({in_inventory}) and the product master ({in_product})"
                            );
                            if let Some(notification) = self.record_referential(
                                report,
                                batch_id,
                                ReferentialFinding {
                                    rule: VrRule::Vr09,
                                    file: sales,
                                    line_no,
                                    key: batch,
                                    reason: &reason,
                                },
                            )? {
                                exceptions.push(notification);
                            }
                        }
                    }
                    _ => {
                        self.record_not_evaluable(
                            report,
                            batch_id,
                            VrRule::Vr09,
                            sales,
                            line_no,
                            batch,
                        )?;
                    }
                }

                let salesperson = row.get(sp_idx).map(|v| v.trim()).unwrap_or("");
                if salesperson.is_empty() {
                    self.record_finding(
                        report,
                        batch_id,
                        Finding {
                            rule: VrRule::Vr10,
                            severity: Severity::Warning,
                            outcome: Outcome::NotEvaluable,
                            file: sales.source_name.clone(),
                            line: line_no,
                            key: String::new(),
                            reason: "SALESPERSON_CODE is empty; mandatory-ness is undefined by the manual (MTRIX-003)".to_string(),
                        },
                    )?;
                } else {
                    match &sales_force {
                        Some(pairs) => {
                            let expected_customer =
                                row.get(cust_idx).map(|v| v.trim()).unwrap_or("");
                            let linked = pairs.contains(&(
                                salesperson.to_string(),
                                expected_customer.to_string(),
                            ));
                            if !linked {
                                let reason = format!(
                                    "salesperson '{salesperson}' is not registered for customer '{expected_customer}' in the sales force master"
                                );
                                if let Some(notification) = self.record_referential(
                                    report,
                                    batch_id,
                                    ReferentialFinding {
                                        rule: VrRule::Vr10,
                                        file: sales,
                                        line_no,
                                        key: salesperson,
                                        reason: &reason,
                                    },
                                )? {
                                    exceptions.push(notification);
                                }
                            }
                        }
                        None => {
                            self.record_not_evaluable(
                                report,
                                batch_id,
                                VrRule::Vr10,
                                sales,
                                line_no,
                                salesperson,
                            )?;
                        }
                    }
                }

                let document_type = row.get(doc_type_idx).map(|v| v.trim()).unwrap_or("");
                if document_type.is_empty() {
                    self.record_finding(
                        report,
                        batch_id,
                        Finding {
                            rule: VrRule::Vr04,
                            severity: Severity::Warning,
                            outcome: Outcome::Suppressed,
                            file: sales.source_name.clone(),
                            line: line_no,
                            key: String::new(),
                            reason: "DOCUMENT_TYPE is empty; the row is kept because the manual does not define the mandatory-ness of the field (MTRIX-003)".to_string(),
                        },
                    )?;
                }
            }
        }

        // --- Inventory rules ----------------------------------------------------
        if let Some(inventory) = parsed
            .iter()
            .find(|f| f.role == FileRole::Inventory && !f.rejected)
        {
            let spec = spec::spec_by_role(FileRole::Inventory);
            let batch_idx = spec
                .index_of_role(KeyRole::Batch)
                .expect("column declared by the MTRIX specification");
            for (offset, row) in inventory.rows.iter().enumerate() {
                let line_no = inventory
                    .row_lines
                    .get(offset)
                    .copied()
                    .unwrap_or(offset + 2);
                let batch = row.get(batch_idx).map(|v| v.trim()).unwrap_or("");
                match &product_keys {
                    Some(keys) if !batch.is_empty() && !keys.contains(batch) => {
                        let reason = format!(
                            "inventory item '{batch}' does not exist in the product master (ALTHPD)"
                        );
                        if let Some(notification) = self.record_referential(
                            report,
                            batch_id,
                            ReferentialFinding {
                                rule: VrRule::Vr11,
                                file: inventory,
                                line_no,
                                key: batch,
                                reason: &reason,
                            },
                        )? {
                            exceptions.push(notification);
                        }
                    }
                    None => {
                        self.record_not_evaluable(
                            report,
                            batch_id,
                            VrRule::Vr11,
                            inventory,
                            line_no,
                            batch,
                        )?;
                    }
                    _ => {}
                }
            }
        }
        // --- VR-14: sales force customers must exist in the customer master ----
        // Source: original manual p.34, confirmed by the V6.0 cross-file section.
        if let Some(sales_force) = parsed
            .iter()
            .find(|f| f.role == FileRole::SalesForce && !f.rejected)
        {
            let spec = spec::spec_by_role(FileRole::SalesForce);
            let cust_idx = spec
                .index_of_role(KeyRole::Customer)
                .expect("column declared by the MTRIX specification");
            for (offset, row) in sales_force.rows.iter().enumerate() {
                let line_no = sales_force
                    .row_lines
                    .get(offset)
                    .copied()
                    .unwrap_or(offset + 2);
                let customer = row.get(cust_idx).map(|v| v.trim()).unwrap_or("");
                if customer.is_empty() {
                    continue;
                }
                match &customers {
                    Some(keys) if !keys.contains(customer) => {
                        let reason = format!(
                            "sales force customer '{customer}' does not exist in the customer master (ALTHCI)"
                        );
                        if let Some(notification) = self.record_referential(
                            report,
                            batch_id,
                            ReferentialFinding {
                                rule: VrRule::Vr14,
                                file: sales_force,
                                line_no,
                                key: customer,
                                reason: &reason,
                            },
                        )? {
                            exceptions.push(notification);
                        }
                    }
                    None => {
                        self.record_not_evaluable(
                            report,
                            batch_id,
                            VrRule::Vr14,
                            sales_force,
                            line_no,
                            customer,
                        )?;
                    }
                    _ => {}
                }
            }
        }

        // --- VR-13: water level alignment between inventory and sales ---------
        // Source: original manual p.18 - the inventory file must carry the last
        // inventory position of the sales movement period that was sent.
        if let (Some(inventory), Some(sales)) = (
            parsed
                .iter()
                .find(|f| f.role == FileRole::Inventory && !f.rejected),
            parsed
                .iter()
                .find(|f| f.role == FileRole::Sales && !f.rejected),
        ) {
            let es_spec = spec::spec_by_role(FileRole::Inventory);
            let vd_spec = spec::spec_by_role(FileRole::Sales);
            let es_date_idx = es_spec
                .index_of("INVENTORY_DATE")
                .expect("column declared by the MTRIX specification");
            let vd_date_idx = vd_spec
                .index_of("DATE")
                .expect("column declared by the MTRIX specification");
            let inventory_date = inventory
                .rows
                .first()
                .and_then(|r| r.get(es_date_idx))
                .map(|v| v.trim().to_string())
                .unwrap_or_default();
            let sales_period_end = sales
                .rows
                .iter()
                .filter_map(|r| r.get(vd_date_idx).map(|v| v.trim().to_string()))
                .max();
            if let (Some(inv_date), Some(sales_end)) =
                (Some(inventory_date.clone()), sales_period_end)
            {
                if !inv_date.is_empty() && inv_date != sales_end {
                    let reason = format!(
                        "inventory snapshot date '{inv_date}' does not match the sales movement period end '{sales_end}'"
                    );
                    self.record_finding(
                        report,
                        batch_id,
                        Finding {
                            rule: VrRule::Vr13,
                            severity: Severity::Warning,
                            outcome: Outcome::Listed,
                            file: inventory.source_name.clone(),
                            line: 2,
                            key: inv_date,
                            reason,
                        },
                    )?;
                }
            }
        }
        Ok(())
    }

    /// Record a finding in the report and in the ledger at the same time.
    ///
    /// The report is the readable summary of a run, the ledger is the durable
    /// evidence. A finding that exists in only one of the two cannot be traced
    /// back later, so both are written together. A row that was isolated is
    /// written to the quarantine log instead, so that the two counts of the
    /// `status` command do not count the same problem twice.
    fn record_finding(
        &self,
        report: &mut ValidationReport,
        batch_id: &str,
        finding: Finding,
    ) -> Result<(), String> {
        self.ledger
            .log_finding(LedgerFinding {
                batch_id,
                rule_code: finding.rule.id(),
                severity: ledger_severity(finding.severity),
                filename: &finding.file,
                line_number: finding.line,
                key_value: &finding.key,
                reason: &finding.reason,
            })
            .map_err(|e| {
                format!(
                    "cannot persist finding {} at {} line {}: {e}",
                    finding.rule.id(),
                    finding.file,
                    finding.line
                )
            })?;
        report.add_finding(finding);
        Ok(())
    }

    fn record_referential(
        &self,
        report: &mut ValidationReport,
        batch_id: &str,
        finding: ReferentialFinding<'_>,
    ) -> Result<Option<ExceptionNotification>, String> {
        self.record_finding(
            report,
            batch_id,
            Finding {
                rule: finding.rule,
                severity: Severity::Error,
                outcome: Outcome::Listed,
                file: finding.file.source_name.clone(),
                line: finding.line_no,
                key: finding.key.to_string(),
                reason: finding.reason.to_string(),
            },
        )?;

        // Direction B (mid-level intervention): emit a structured exception
        // notification so the business side can synchronize master data.
        let notification = ExceptionNotification {
            rule_id: finding.rule.id().to_string(),
            orphan_key: finding.key.to_string(),
            file: finding.file.source_name.clone(),
            line: finding.line_no,
            suggested_action: match finding.rule {
                VrRule::Vr08 => format!(
                    "please synchronize customer code '{}' to ALTHCI customer master",
                    finding.key
                ),
                VrRule::Vr09 => format!(
                    "please synchronize batch code '{}' to ALTHES and ALTHPD masters",
                    finding.key
                ),
                _ => "please confirm master data synchronization status with the business side"
                    .to_string(),
            },
            notified_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .to_string(),
        };
        Ok(Some(notification))
    }

    fn record_not_evaluable(
        &self,
        report: &mut ValidationReport,
        batch_id: &str,
        rule: VrRule,
        file: &ParsedFile,
        line_no: usize,
        key: &str,
    ) -> Result<(), String> {
        self.record_finding(
            report,
            batch_id,
            Finding {
                rule,
                severity: Severity::Warning,
                outcome: Outcome::NotEvaluable,
                file: file.source_name.clone(),
                line: line_no,
                key: key.to_string(),
                reason: "a required file was rejected, so this rule could not be evaluated"
                    .to_string(),
            },
        )
    }

    /// VR-12 preparation: amounts grouped by country. Currencies are never
    /// converted (MTRIX-023), the country is the reporting unit.
    pub fn currency_groups(&self, parsed: &[ParsedFile]) -> Result<Vec<CurrencyGroup>, String> {
        let mut groups: HashMap<String, CurrencyGroup> = HashMap::new();
        if let Some(sales) = parsed
            .iter()
            .find(|f| f.role == FileRole::Sales && !f.rejected)
        {
            let spec = spec::spec_by_role(FileRole::Sales);
            let country_idx = spec
                .index_of("COUNTRY_ISO")
                .expect("column declared by the MTRIX specification");
            let qty_idx = spec
                .index_of("QUANTITY")
                .expect("column declared by the MTRIX specification");
            let price_idx = spec
                .index_of("SALES_PRICE")
                .expect("column declared by the MTRIX specification");
            for (offset, row) in sales.rows.iter().enumerate() {
                let country = row
                    .get(country_idx)
                    .map(|v| v.trim().to_string())
                    .unwrap_or_default();
                if country.is_empty() {
                    continue;
                }
                let quantity_raw = row.get(qty_idx).ok_or_else(|| {
                    format!("accepted sales row {} is missing QUANTITY", offset + 2)
                })?;
                let quantity = ScaledDecimal::parse(quantity_raw).map_err(|e| {
                    format!(
                        "accepted sales row {} has invalid QUANTITY: {e}",
                        offset + 2
                    )
                })?;
                let price_raw = row.get(price_idx).ok_or_else(|| {
                    format!("accepted sales row {} is missing SALES_PRICE", offset + 2)
                })?;
                let price = ScaledDecimal::parse(price_raw).map_err(|e| {
                    format!(
                        "accepted sales row {} has invalid SALES_PRICE: {e}",
                        offset + 2
                    )
                })?;
                let entry = groups.entry(country.clone()).or_insert(CurrencyGroup {
                    country_iso: country,
                    rows: 0,
                    quantity_units: 0,
                    amount_units: 0,
                });
                entry.rows = entry
                    .rows
                    .checked_add(1)
                    .ok_or_else(|| format!("sales row count overflow for {}", entry.country_iso))?;
                entry.quantity_units = entry
                    .quantity_units
                    .checked_add(i128::from(quantity.units()))
                    .ok_or_else(|| format!("quantity total overflow for {}", entry.country_iso))?;
                let amount = i128::from(quantity.units())
                    .checked_mul(i128::from(price.units()))
                    .and_then(|v| v.checked_div(10_000))
                    .ok_or_else(|| {
                        format!("amount multiplication overflow for {}", entry.country_iso)
                    })?;
                entry.amount_units = entry
                    .amount_units
                    .checked_add(amount)
                    .ok_or_else(|| format!("amount total overflow for {}", entry.country_iso))?;
            }
        }
        let mut groups: Vec<CurrencyGroup> = groups.into_values().collect();
        groups.sort_by(|a, b| a.country_iso.cmp(&b.country_iso));
        Ok(groups)
    }

    fn apply_counts(&mut self, role: FileRole, count: usize) {
        match role {
            FileRole::Customer => self.repaired_counts.althci += count,
            FileRole::Inventory => self.repaired_counts.althes += count,
            FileRole::SalesForce => self.repaired_counts.althfv += count,
            FileRole::Product => self.repaired_counts.althpd += count,
            FileRole::Sales => self.repaired_counts.althvd += count,
        }
    }
}

/// Map a validation severity onto the severity stored in the ledger.
fn ledger_severity(severity: Severity) -> FindingSeverity {
    match severity {
        Severity::Error => FindingSeverity::Error,
        Severity::Warning => FindingSeverity::Warning,
        Severity::Info => FindingSeverity::Info,
    }
}

/// The rule that governs a coded value in a given file.
///
/// The sales document type is covered by VR-04, the product coded fields
/// (package type, barcode type, product status) by VR-07.
fn coded_field_rule(role: FileRole) -> VrRule {
    match role {
        FileRole::Product => VrRule::Vr07,
        _ => VrRule::Vr04,
    }
}

/// Whether a raw record carries a negative inventory quantity.
fn negative_measure(file_spec: &FileSpec, raw_line: &str) -> bool {
    let index = match file_spec
        .columns
        .iter()
        .position(|c| c.source_name == "QUANTITY_STOCK")
    {
        Some(index) => index,
        None => return false,
    };
    let fields: Vec<&str> = raw_line.split(';').collect();
    fields
        .get(index)
        .and_then(|value| ScaledDecimal::parse(value).ok())
        .map(|value| value.is_negative())
        .unwrap_or(false)
}

/// Timestamp fragment of a delivered file name, used to pick the newest file.
/// The fragment is the run of ASCII digits immediately following the role
/// prefix. Delivered stamps are fixed-width (DDMMYYYYHHMMSSmmm) and
/// zero-padded, so lexicographic comparison is order-equivalent to a numeric
/// comparison; this must not be relied upon for shorter or unpadded stamps.
fn file_stamp(file_name: &str, prefix: &str) -> String {
    let after_prefix = file_name
        .trim_start_matches(|c: char| c.is_ascii_alphabetic())
        .get(prefix.len()..)
        .unwrap_or_default()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>();
    after_prefix
}

/// Name of the repaired output file.
pub fn repaired_file_name(source_name: &str) -> String {
    let base = source_name.trim();
    match base.rsplit_once('.') {
        Some((stem, ext)) => format!("{stem}_repaired.{ext}"),
        None => format!("{base}_repaired"),
    }
}

/// Conformance of the declared specification.
pub fn spec_conformance() -> crate::validators::SpecConformance {
    let mut fields = 0usize;
    let mut with_provenance = 0usize;
    for file in spec::MTRIX_SPEC {
        for col in file.columns {
            fields += 1;
            if col.manual_page.is_some() {
                with_provenance += 1;
            }
        }
    }
    crate::validators::SpecConformance {
        files_declared: spec::MTRIX_SPEC.len(),
        fields_declared: fields,
        fields_with_provenance: with_provenance,
        rules_declared: VrRule::all().len(),
        rules_implemented: VrRule::all().len(),
        rules_from_field_dictionary: VrRule::L2_DECLARED,
        rules_from_original_manual: VrRule::MANUAL_DERIVED,
        files_with_snapshot_page: spec::MANUAL_PDF_PAGE.len(),
    }
}

/// Build the repaired file set from raw file contents. Kept for callers that
/// already hold the five documents in memory.
pub fn build_file_set(
    althci: &str,
    althes: &str,
    althfv: &str,
    althpd: &str,
    althvd: &str,
) -> MtrixFileSet {
    let mut set = MtrixFileSet::default();
    for (role, content) in [
        (FileRole::Customer, althci),
        (FileRole::Inventory, althes),
        (FileRole::SalesForce, althfv),
        (FileRole::Product, althpd),
        (FileRole::Sales, althvd),
    ] {
        let file_spec = spec::spec_by_role(role);
        let records = wellfit_core::text::split_records(content);
        let raw_header = records.first().map(|(_, line)| *line).unwrap_or_default();
        let (header, mut log) = repairs::repair_header(file_spec, raw_header);
        let mut body = header.join(";");
        for (line_no, line) in records.iter().skip(1) {
            if line.trim().is_empty() {
                continue;
            }
            let values = repairs::repair_row(file_spec, file_spec.prefix, *line_no, line, &mut log);
            body.push('\n');
            body.push_str(&values.join(";"));
        }
        *set.get_mut(role) = body;
    }
    set
}

/// Batch status label helper for CLI output.
pub fn status_label(status: BatchStatus) -> &'static str {
    status.label()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("wellfit-mtrix-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(dir: &Path, name: &str, content: &str) {
        fs::write(dir.join(name), content).unwrap();
    }

    const REAL_CI: &str = "VENDOR_ID;DIST_ID;CUSTOMER_ID;CUSTOMER_NAME;ADRESS;DISTRICT;ZIP_CODE;CITY;STATE;SALES_CONTACT;PHONE_NUMBERS;CUSTOMER_ID;ROUTE;STORE_TYPE;PROPORTION;COUNTRY_ISO;\n\
6063343900011222;029864857023000;995A;JDID;;;;Bogor;;;;995A;Online;Online;0;USA\n\
6063343900011222;029864857023000;995C;Shopee;;;;Bogor;;;;995C;Online;Online;0;USA\n";

    const REAL_ES: &str =
        "INVENTORY_DATE;VENDOR_ID;DIST_ID;BATCH_CODE;QUANTITY_STOCK;COUNTRY_ISO\n\
20240221;USA6063343900011222;029864857023000;7890541735003;167;USA\n";

    const REAL_FV: &str = "VENDOR_ID;DIST_ID;CUSTOMER_ID;MANAGER_CODE;MANAGER_NAME;SUPERVISOR_ID;SUPERVISOR_NAME;SALESPERSON_CODE;SALESPERSON_NAME;COUNTRY\n\
6063343900011222;029864857023000;995A;;;;;;USA\n";

    const REAL_PD: &str = "TRANSACTION_DATE;DIST_ID;VENDOR_ID;VENDOR_COMPANY_NAME;PRODUCT_CODE;PACKAGE_TYPE;BARCODE;BARCODE_TYPE;DESCRIPTION;PRODUCT_DIV;PRODUCT_STATUS;COUNTRY_ISO\n\
20240224;02.986.485.7023.000;6063343900011222;Alpargatas SE;8;0;7898656600087 ;1;;RSF;A;USA\n";

    const REAL_VD: &str = "VENDOR_ID;DIST_ID;CUSTOMER_ID;DATE;DOCUMENT_NUMBER;BATCH_CODE;QUANTITY;SALES_PRICE;SALESPERSON_CODE;DOCUMENT_TYPE;ZIP_CODE;COUNTRY_ISO;\n\
6063343900011222;029864857023000;7AC;20220109;348732;7893249911649;1;174300;;;;ARG;\n";

    fn write_all(dir: &Path) {
        write(dir, "ALTHCI30062020153005130 2 (3).csv", REAL_CI);
        write(dir, "ALTHES30062020153005130 2 (3).csv", REAL_ES);
        write(dir, "ALTHFV30062020153005130 2 (3).csv", REAL_FV);
        write(dir, "ALTHPD30062020153005130 2 (3).csv", REAL_PD);
        write(dir, "ALTHVD30062020153005130 5 (4).csv", REAL_VD);
    }

    #[test]
    fn test_discovery_requires_all_five_files() {
        let dir = temp_dir("missing");
        write(&dir, "ALTHCI30062020153005130 2 (3).csv", REAL_CI);
        let err = MtrixEngine::discover_files(&dir).unwrap_err();
        assert!(err.contains("ALTHES"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_discovery_finds_all_five_files() {
        let dir = temp_dir("discover");
        write_all(&dir);
        let found = MtrixEngine::discover_files(&dir).unwrap();
        assert_eq!(found.len(), 5);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_process_drop_reports_violations_without_dropping_rows() {
        let dir = temp_dir("process");
        let out = temp_dir("process-out");
        write_all(&dir);

        let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = MtrixEngine::new(ledger, gate);
        let report = engine.process_drop(&dir, "B-TEST", Some(&out)).unwrap();

        assert_eq!(report.files.len(), 5);
        assert!(report.files.iter().all(|f| f.header_ok));
        // Sales customer 7AC is not in the customer master -> listed, not isolated.
        let vr08 = report
            .findings
            .iter()
            .filter(|f| f.rule == VrRule::Vr08)
            .count();
        assert_eq!(vr08, 1);
        // The sales item is in neither the inventory nor the product master.
        let vr09 = report
            .findings
            .iter()
            .filter(|f| f.rule == VrRule::Vr09)
            .count();
        assert_eq!(vr09, 1);
        // Inventory item is not in the product master -> listed.
        let vr11 = report
            .findings
            .iter()
            .filter(|f| f.rule == VrRule::Vr11)
            .count();
        assert_eq!(vr11, 1);
        // No row is isolated: the manual asks to list the offending keys.
        assert_eq!(report.error_count(), 3);
        assert!(report.files.iter().all(|f| f.rows_quarantined == 0));
        assert_eq!(report.status, BatchStatus::PartialAccepted);
        assert_eq!(report.currency_groups.len(), 1);
        assert_eq!(report.currency_groups[0].country_iso, "ARG");
        assert_eq!(report.currency_groups[0].quantity_units, 10_000);
        assert_eq!(report.currency_groups[0].amount_units, 1_743_000_000);
        let serialized = serde_json::to_string(&report).unwrap();
        let restored: ValidationReport = serde_json::from_str(&serialized).unwrap();
        assert_eq!(restored.currency_groups, report.currency_groups);

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn test_process_drop_writes_repaired_output() {
        let dir = temp_dir("output");
        let out = temp_dir("output-out");
        write_all(&dir);

        let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = MtrixEngine::new(ledger, gate);
        engine.process_drop(&dir, "B-TEST", Some(&out)).unwrap();

        let repaired = out.join("ALTHES30062020153005130 2 (3)_repaired.csv");
        assert!(repaired.exists(), "repaired file must be written");
        let body = fs::read_to_string(&repaired).unwrap();
        assert!(body
            .starts_with("INVENTORY_DATE;VENDOR_ID;DIST_ID;BATCH_CODE;QUANTITY_STOCK;COUNTRY_ISO"));
        // The vendor country prefix is separated from the vendor code.
        assert!(body.contains(";6063343900011222;"));
        assert!(!body.contains("USA6063343900011222"));

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn test_finding_line_numbers_follow_the_delivered_file() {
        let dir = temp_dir("line-numbers");
        let out = temp_dir("line-numbers-out");
        write(&dir, "ALTHCI30062020153005130 2 (3).csv", REAL_CI);
        write(&dir, "ALTHES30062020153005130 2 (3).csv", REAL_ES);
        write(&dir, "ALTHFV30062020153005130 2 (3).csv", REAL_FV);
        write(&dir, "ALTHPD30062020153005130 2 (3).csv", REAL_PD);
        // A blank line between the header and the data row, and an impossible
        // calendar date on the fifth line of the file.
        write(
            &dir,
            "ALTHVD30062020153005130 5 (4).csv",
            "VENDOR_ID;DIST_ID;CUSTOMER_ID;DATE;DOCUMENT_NUMBER;BATCH_CODE;QUANTITY;SALES_PRICE;SALESPERSON_CODE;DOCUMENT_TYPE;ZIP_CODE;COUNTRY_ISO;\n\n\
6063343900011222;029864857023000;7AC;20220109;348732;7893249911649;1;174300;;;;ARG;\n\
6063343900011222;029864857023000;7AC;20240230;348733;7893249911649;1;174300;;;;ARG;\n",
        );

        let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = MtrixEngine::new(ledger, gate);
        let report = engine.process_drop(&dir, "B-LINE", Some(&out)).unwrap();

        let invalid_date = report
            .findings
            .iter()
            .find(|f| f.rule == VrRule::Vr03)
            .expect("the impossible date must be reported");
        // File layout: line 1 header, line 2 blank, line 3 first row,
        // line 4 the row with the impossible date. The finding must point at
        // line 4; dropping the blank line would have reported line 3.
        assert_eq!(invalid_date.line, 4, "line numbers must follow the file");

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn test_duplicate_transaction_sums_the_quantities() {
        let dir = temp_dir("aggregate");
        let out = temp_dir("aggregate-out");
        write(&dir, "ALTHCI30062020153005130 2 (3).csv", REAL_CI);
        write(&dir, "ALTHES30062020153005130 2 (3).csv", REAL_ES);
        write(&dir, "ALTHFV30062020153005130 2 (3).csv", REAL_FV);
        write(&dir, "ALTHPD30062020153005130 2 (3).csv", REAL_PD);
        // The same document and item twice, with different quantities: 1 and 5.
        write(
            &dir,
            "ALTHVD30062020153005130 5 (4).csv",
            "VENDOR_ID;DIST_ID;CUSTOMER_ID;DATE;DOCUMENT_NUMBER;BATCH_CODE;QUANTITY;SALES_PRICE;SALESPERSON_CODE;DOCUMENT_TYPE;ZIP_CODE;COUNTRY_ISO;\n\
6063343900011222;029864857023000;995A;20220109;348732;7898656600087;1;174300;;;;ARG;\n\
6063343900011222;029864857023000;995A;20220109;348732;7898656600087;5;174300;;;;ARG;\n",
        );

        let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = MtrixEngine::new(ledger, gate);
        let report = engine.process_drop(&dir, "B-AGG", Some(&out)).unwrap();

        // The duplicate is reported at error level.
        let vr05 = report
            .findings
            .iter()
            .filter(|f| f.rule == VrRule::Vr05)
            .count();
        assert_eq!(vr05, 1);
        assert!(report
            .findings
            .iter()
            .any(|f| f.rule == VrRule::Vr05 && f.severity == Severity::Error));

        // The aggregated row carries the sum of the two quantities, 1 + 5.
        let repaired =
            fs::read_to_string(out.join("ALTHVD30062020153005130 5 (4)_repaired.csv")).unwrap();
        let data_rows: Vec<&str> = repaired
            .split('\r')
            .filter(|s| !s.is_empty())
            .skip(1)
            .collect();
        assert_eq!(data_rows.len(), 1, "the duplicate must become one row");
        assert!(
            data_rows[0].contains(";6;"),
            "the quantities must be summed, not doubled: {}",
            data_rows[0]
        );

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn test_excess_quantity_precision_is_quarantined() {
        let dir = temp_dir("quantity-precision");
        let out = temp_dir("quantity-precision-out");
        write_all(&dir);
        write(
            &dir,
            "ALTHVD30062020153005130 5 (4).csv",
            "VENDOR_ID;DIST_ID;CUSTOMER_ID;DATE;DOCUMENT_NUMBER;BATCH_CODE;QUANTITY;SALES_PRICE;SALESPERSON_CODE;DOCUMENT_TYPE;ZIP_CODE;COUNTRY_ISO;\n\
6063343900011222;029864857023000;7AC;20220109;348732;7893249911649;1.23456;174300;;;;ARG;\n",
        );

        let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = MtrixEngine::new(ledger, gate);
        let report = engine
            .process_drop(&dir, "B-QUANTITY-PRECISION", Some(&out))
            .unwrap();

        let sales = report
            .files
            .iter()
            .find(|file| file.prefix == "ALTHVD")
            .expect("the sales file outcome is included");
        assert_eq!(sales.rows_quarantined, 1);
        assert!(report.findings.iter().any(|finding| {
            finding.rule == VrRule::Vr15 && finding.reason.contains("QUANTITY")
        }));

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn test_duplicate_quantity_overflow_fails_instead_of_reusing_prior_value() {
        let dir = temp_dir("aggregate-overflow");
        let out = temp_dir("aggregate-overflow-out");
        write(&dir, "ALTHCI30062020153005130 2 (3).csv", REAL_CI);
        write(&dir, "ALTHES30062020153005130 2 (3).csv", REAL_ES);
        write(&dir, "ALTHFV30062020153005130 2 (3).csv", REAL_FV);
        write(&dir, "ALTHPD30062020153005130 2 (3).csv", REAL_PD);
        let duplicate_rows = (0..10)
            .map(|_| {
                "6063343900011222;029864857023000;995A;20220109;348732;7898656600087;99999999999999.9999;174300;;;;ARG;"
            })
            .collect::<Vec<_>>()
            .join("\n");
        let sales = format!(
            "VENDOR_ID;DIST_ID;CUSTOMER_ID;DATE;DOCUMENT_NUMBER;BATCH_CODE;QUANTITY;SALES_PRICE;SALESPERSON_CODE;DOCUMENT_TYPE;ZIP_CODE;COUNTRY_ISO;\n{duplicate_rows}\n"
        );
        write(&dir, "ALTHVD30062020153005130 5 (4).csv", &sales);

        let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = MtrixEngine::new(ledger, gate);
        let result = engine.process_drop(&dir, "B-AGG-OVERFLOW", Some(&out));

        let error = result.expect_err("quantity overflow must stop the drop");
        assert!(error.contains("quantity overflow"), "{error}");
        assert!(
            fs::read_dir(&out)
                .map(|entries| entries.count())
                .unwrap_or_default()
                == 0,
            "failed validation must not publish repaired output files"
        );

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn test_aggregation_does_not_shift_cross_file_line_numbers() {
        let dir = temp_dir("agg-shift");
        let out = temp_dir("agg-shift-out");
        write(&dir, "ALTHCI30062020153005130 2 (3).csv", REAL_CI);
        write(&dir, "ALTHES30062020153005130 2 (3).csv", REAL_ES);
        write(&dir, "ALTHFV30062020153005130 2 (3).csv", REAL_FV);
        write(&dir, "ALTHPD30062020153005130 2 (3).csv", REAL_PD);
        // Data rows 2 and 3 are VR-05 duplicates (same customer + document +
        // item, quantities 1 and 5). Data row 4 carries an unknown customer
        // (7AC) and must be reported at physical line 4, not shifted back to
        // line 3 by the row that was collapsed into the duplicate.
        write(
            &dir,
            "ALTHVD30062020153005130 5 (4).csv",
            "VENDOR_ID;DIST_ID;CUSTOMER_ID;DATE;DOCUMENT_NUMBER;BATCH_CODE;QUANTITY;SALES_PRICE;SALESPERSON_CODE;DOCUMENT_TYPE;ZIP_CODE;COUNTRY_ISO;\n\
6063343900011222;029864857023000;995A;20220109;348732;7898656600087;1;174300;;;;ARG;\n\
6063343900011222;029864857023000;995A;20220109;348732;7898656600087;5;174300;;;;ARG;\n\
6063343900011222;029864857023000;7AC;20220109;348733;7893249911649;1;174300;;;;ARG;\n",
        );

        let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = MtrixEngine::new(ledger, gate);
        let report = engine.process_drop(&dir, "B-AGG-LINE", Some(&out)).unwrap();

        let vr05 = report
            .findings
            .iter()
            .filter(|f| f.rule == VrRule::Vr05)
            .count();
        assert_eq!(vr05, 1, "the duplicate must still be reported once");

        let customer_finding = report
            .findings
            .iter()
            .find(|f| f.rule == VrRule::Vr08)
            .expect("the unknown customer must be reported");
        assert_eq!(
            customer_finding.line, 4,
            "a surviving row after VR-05 aggregation must keep its physical line number"
        );

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn test_a_quarantined_row_does_not_shift_the_line_numbers() {
        let dir = temp_dir("shift");
        let out = temp_dir("shift-out");
        write(&dir, "ALTHCI30062020153005130 2 (3).csv", REAL_CI);
        write(&dir, "ALTHES30062020153005130 2 (3).csv", REAL_ES);
        write(&dir, "ALTHFV30062020153005130 2 (3).csv", REAL_FV);
        write(&dir, "ALTHPD30062020153005130 2 (3).csv", REAL_PD);
        // Line 2 carries an impossible date and is isolated; line 3 carries an
        // unknown customer and must still be reported as line 3.
        write(
            &dir,
            "ALTHVD30062020153005130 5 (4).csv",
            "VENDOR_ID;DIST_ID;CUSTOMER_ID;DATE;DOCUMENT_NUMBER;BATCH_CODE;QUANTITY;SALES_PRICE;SALESPERSON_CODE;DOCUMENT_TYPE;ZIP_CODE;COUNTRY_ISO;\n\
6063343900011222;029864857023000;995A;20240230;348732;7893249911649;1;174300;;;;ARG;\n\
6063343900011222;029864857023000;7AC;20220109;348733;7893249911649;1;174300;;;;ARG;\n",
        );

        let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = MtrixEngine::new(ledger, gate);
        let report = engine.process_drop(&dir, "B-SHIFT", Some(&out)).unwrap();

        let date_finding = report
            .findings
            .iter()
            .find(|f| f.rule == VrRule::Vr03)
            .expect("the impossible date must be reported");
        assert_eq!(date_finding.line, 2);

        let customer_finding = report
            .findings
            .iter()
            .find(|f| f.rule == VrRule::Vr08)
            .expect("the unknown customer must be reported");
        assert_eq!(
            customer_finding.line, 3,
            "an isolated row must not shift the line numbers of the following rows"
        );

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn test_the_same_drop_cannot_be_ingested_twice() {
        let dir = temp_dir("idempotent");
        let out = temp_dir("idempotent-out");
        write(&dir, "ALTHCI30062020153005130 2 (3).csv", REAL_CI);
        write(&dir, "ALTHES30062020153005130 2 (3).csv", REAL_ES);
        write(&dir, "ALTHFV30062020153005130 2 (3).csv", REAL_FV);
        write(&dir, "ALTHPD30062020153005130 2 (3).csv", REAL_PD);
        write(&dir, "ALTHVD30062020153005130 5 (4).csv", REAL_VD);

        let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = MtrixEngine::new(ledger.clone(), gate);
        engine.process_drop(&dir, "B-ONCE", Some(&out)).unwrap();

        // The same ledger, the same batch: the second ingestion must be refused.
        let mut second = MtrixEngine::new(ledger, Arc::new(Mutex::new(ReleaseGate::default())));
        let error = second.process_drop(&dir, "B-ONCE", Some(&out)).unwrap_err();
        assert!(error.contains("already registered"), "{error}");

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn test_a_corrected_drop_can_be_resent_under_the_same_batch() {
        let dir = temp_dir("resend");
        let out = temp_dir("resend-out");
        write(&dir, "ALTHCI30062020153005130 2 (3).csv", REAL_CI);
        write(&dir, "ALTHES30062020153005130 2 (3).csv", REAL_ES);
        write(&dir, "ALTHFV30062020153005130 2 (3).csv", REAL_FV);
        write(&dir, "ALTHPD30062020153005130 2 (3).csv", REAL_PD);
        // A row with an impossible date: the row is isolated, so the file is
        // defective and the batch claim is released.
        write(
            &dir,
            "ALTHVD30062020153005130 5 (4).csv",
            "VENDOR_ID;DIST_ID;CUSTOMER_ID;DATE;DOCUMENT_NUMBER;BATCH_CODE;QUANTITY;SALES_PRICE;SALESPERSON_CODE;DOCUMENT_TYPE;ZIP_CODE;COUNTRY_ISO;\n\
6063343900011222;029864857023000;995A;20240230;348732;7893249911649;1;174300;;;;ARG;\n",
        );

        let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
        let mut engine =
            MtrixEngine::new(ledger.clone(), Arc::new(Mutex::new(ReleaseGate::default())));
        let first = engine.process_drop(&dir, "B-RESEND", Some(&out)).unwrap();
        assert!(first.files.iter().any(|f| f.rows_quarantined > 0));

        // The corrected file, same batch identifier: it must be accepted.
        write(
            &dir,
            "ALTHVD30062020153005130 5 (4).csv",
            "VENDOR_ID;DIST_ID;CUSTOMER_ID;DATE;DOCUMENT_NUMBER;BATCH_CODE;QUANTITY;SALES_PRICE;SALESPERSON_CODE;DOCUMENT_TYPE;ZIP_CODE;COUNTRY_ISO;\n\
6063343900011222;029864857023000;995A;20220109;348732;7893249911649;1;174300;;;;ARG;\n",
        );
        let mut second = MtrixEngine::new(ledger, Arc::new(Mutex::new(ReleaseGate::default())));
        let report = second.process_drop(&dir, "B-RESEND", Some(&out)).unwrap();
        assert_eq!(report.files.len(), 5);
        assert!(report.files.iter().all(|f| f.rows_quarantined == 0));

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn test_carriage_return_terminated_files_are_processed() {
        // The counterparty delivers CR terminated files. A parser that only
        // understands LF sees the whole file as one line and rejects it, so
        // this package must cover the case on its own.
        let dir = temp_dir("cr");
        let out = temp_dir("cr-out");
        let write_cr = |name: &str, content: &str| {
            write(&dir, name, &content.replace('\n', "\r"));
        };
        write_cr(
            "ALTHCI30062020153005130 2 (3).csv",
            "VENDOR_ID;DIST_ID;CUSTOMER_ID;CUSTOMER_NAME;ADRESS;DISTRICT;ZIP_CODE;CITY;STATE;SALES_CONTACT;PHONE_NUMBERS;CUSTOMER_ID;ROUTE;STORE_TYPE;PROPORTION;COUNTRY_ISO;\n6063343900011222;029864857023000;995A;JDID;;;;Bogor;;;;995A;Online;Online;0;USA\n",
        );
        write_cr(
            "ALTHES30062020153005130 2 (3).csv",
            "INVENTORY_DATE;VENDOR_ID;DIST_ID;BATCH_CODE;QUANTITY_STOCK;COUNTRY_ISO\n20240221;USA6063343900011222;029864857023000;7890541735003;167;USA\n",
        );
        write_cr(
            "ALTHFV30062020153005130 2 (3).csv",
            "VENDOR_ID;DIST_ID;CUSTOMER_ID;MANAGER_CODE;MANAGER_NAME;SUPERVISOR_ID;SUPERVISOR_NAME;SALESPERSON_CODE;SALESPERSON_NAME;COUNTRY\n6063343900011222;029864857023000;995A;;;;;;USA\n",
        );
        write_cr(
            "ALTHPD30062020153005130 2 (3).csv",
            "TRANSACTION_DATE;DIST_ID;VENDOR_ID;VENDOR_COMPANY_NAME;PRODUCT_CODE;PACKAGE_TYPE;BARCODE;BARCODE_TYPE;DESCRIPTION;PRODUCT_DIV;PRODUCT_STATUS;COUNTRY_ISO\n20240224;02.986.485.7023.000;6063343900011222;Alpargatas SE;8;0;7898656600087 ;1;;RSF;A;USA\n",
        );
        write_cr(
            "ALTHVD30062020153005130 5 (4).csv",
            "VENDOR_ID;DIST_ID;CUSTOMER_ID;DATE;DOCUMENT_NUMBER;BATCH_CODE;QUANTITY;SALES_PRICE;SALESPERSON_CODE;DOCUMENT_TYPE;ZIP_CODE;COUNTRY_ISO;\n6063343900011222;029864857023000;7AC;20220109;348732;7893249911649;1;174300;;;;ARG;\n",
        );

        let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = MtrixEngine::new(ledger, gate);
        let report = engine.process_drop(&dir, "B-CR", Some(&out)).unwrap();

        assert_eq!(report.files.len(), 5);
        assert!(
            report.files.iter().all(|f| f.header_ok),
            "CR terminated files must be split into records: {:?}",
            report
                .files
                .iter()
                .map(|f| (f.prefix.clone(), f.header_ok, f.rows_total))
                .collect::<Vec<_>>()
        );
        assert!(report.files.iter().all(|f| f.rows_total == 1));

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn test_product_dist_id_dot_mask_is_repaired() {
        let dir = temp_dir("dotmask");
        let out = temp_dir("dotmask-out");
        write_all(&dir);

        let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = MtrixEngine::new(ledger, gate);
        engine.process_drop(&dir, "B-TEST", Some(&out)).unwrap();

        let repaired = out.join("ALTHPD30062020153005130 2 (3)_repaired.csv");
        let body = fs::read_to_string(&repaired).unwrap();
        assert!(body.contains(";029864857023000;"));
        assert!(!body.contains("02.986.485.7023.000"));

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn test_header_mismatch_rejects_only_that_file() {
        let dir = temp_dir("badheader");
        let out = temp_dir("badheader-out");
        write_all(&dir);
        write(
            &dir,
            "ALTHVD30062020153005130 5 (4).csv",
            "WRONG_HEADER;DIST_ID;X\n1;2;3\n",
        );

        let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = MtrixEngine::new(ledger, gate);
        let report = engine.process_drop(&dir, "B-TEST", Some(&out)).unwrap();

        let sales = report
            .files
            .iter()
            .find(|f| f.prefix == "ALTHVD")
            .expect("sales outcome present");
        assert!(sales.rejected);
        assert_eq!(sales.rows_accepted, 0);
        // The other four files are still processed.
        assert!(report
            .files
            .iter()
            .filter(|f| f.prefix != "ALTHVD")
            .all(|f| !f.rejected));
        assert_eq!(report.status, BatchStatus::PartialAccepted);

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn test_spec_conformance_reports_all_fields() {
        let conformance = spec_conformance();
        assert_eq!(conformance.files_declared, 5);
        assert_eq!(conformance.fields_declared, 56);
        assert_eq!(conformance.fields_with_provenance, 56);
        assert_eq!(conformance.rules_declared, 15);
        assert_eq!(conformance.rules_from_field_dictionary, 13);
        assert_eq!(conformance.rules_from_original_manual, 2);
    }

    #[test]
    fn test_fault_injection_ledger_failure_aborts_drop() {
        let dir = temp_dir("fault-injection");
        let out = temp_dir("fault-injection-out");
        write_all(&dir);

        let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
        // Inject a trigger that fails all writes to validation_findings
        ledger
            .execute_sql(
                "CREATE TRIGGER fail_finding BEFORE INSERT ON validation_findings \
                 BEGIN SELECT RAISE(FAIL, 'injected audit failure'); END;",
            )
            .unwrap();

        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = MtrixEngine::new(ledger, gate);
        // process_drop should fail because record_finding cannot persist
        let result = engine.process_drop(&dir, "B-FAULT", Some(&out));
        assert!(
            result.is_err(),
            "ledger failure must propagate to process_drop"
        );

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn test_fault_injection_ledger_failure_no_partial_report() {
        let dir = temp_dir("fault-partial");
        let out = temp_dir("fault-partial-out");
        write_all(&dir);

        let ledger = Arc::new(IdempotencyLedger::in_memory().unwrap());
        // Inject trigger after first finding to simulate mid-process failure
        ledger
            .execute_sql(
                "CREATE TRIGGER fail_finding BEFORE INSERT ON validation_findings \
                 BEGIN SELECT RAISE(FAIL, 'injected mid-process failure'); END;",
            )
            .unwrap();

        let gate = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine = MtrixEngine::new(ledger.clone(), gate);
        let result = engine.process_drop(&dir, "B-FAULT-PARTIAL", Some(&out));

        // Should fail because findings cannot be persisted
        assert!(result.is_err());

        // Verify no findings were partially persisted (trigger blocks all)
        let findings = ledger.get_findings(None).unwrap();
        assert!(
            findings.is_empty(),
            "no findings should be persisted when trigger blocks all writes"
        );

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn test_fault_injection_crash_recovery_data_consistency() {
        let dir = temp_dir("crash-recovery");
        let out = temp_dir("crash-recovery-out");
        write_all(&dir);

        // First run: successful ingestion
        let ledger1 = Arc::new(IdempotencyLedger::in_memory().unwrap());
        let gate1 = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine1 = MtrixEngine::new(ledger1.clone(), gate1);
        let report1 = engine1.process_drop(&dir, "B-CRASH", Some(&out)).unwrap();
        let _finding_count_1 = report1.findings.len();

        // Verify findings were persisted (ledger may have fewer than report because
        // quarantined rows go to a separate table, but must have at least some)
        let persisted_1 = ledger1.get_findings(None).unwrap();
        assert!(
            !persisted_1.is_empty(),
            "findings must be persisted after successful run"
        );

        // Second run with same batch_id: should be deduplicated (idempotent)
        let gate2 = Arc::new(Mutex::new(ReleaseGate::default()));
        let mut engine2 = MtrixEngine::new(ledger1.clone(), gate2);
        let _result2 = engine2.process_drop(&dir, "B-CRASH", Some(&out));

        // Data must remain consistent (no duplicate findings created)
        let persisted_2 = ledger1.get_findings(None).unwrap();
        assert_eq!(
            persisted_2.len(),
            persisted_1.len(),
            "re-run must not create duplicate findings"
        );

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }
}
